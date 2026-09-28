use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::backend::audio::SoundHandle;
use crate::binary_data::BinaryData;
use crate::display_object::{
    Avm1Button, Avm2Button, BitmapClass, EditText, Graphic, MorphShape, MovieClip, Text, Video,
};
use crate::font::Font;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{Bitmap as RenderBitmap, BitmapHandle, BitmapSize};
use ruffle_render::error::Error as RenderError;
use swf::DefineBitsLossless;

// O RELOGIO DAS TEXTURAS.
//
// So precisa de uma coisa: dizer, em segundos, ha quanto tempo o jogo esta
// aberto. Quem desenha uma imagem carimba este numero nela; a varredura
// compara o carimbo com o numero de agora. Nada disso precisa ser exato.
//
// Um numero solto, em vez de passar o tempo por parametro, porque desenhar
// acontece em dezenas de lugares e nenhum deles tem motivo pra saber que
// existe uma varredura.
static RELOGIO: AtomicU64 = AtomicU64::new(0);

/// Avisa que tantos segundos se passaram desde que o jogo abriu.
pub fn marcar_o_tempo(segundos: u64) {
    RELOGIO.store(segundos, Ordering::Relaxed);
}

pub fn agora() -> u64 {
    RELOGIO.load(Ordering::Relaxed)
}

#[derive(Copy, Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Character<'gc> {
    EditText(EditText<'gc>),
    Graphic(Graphic<'gc>),
    MovieClip(MovieClip<'gc>),
    Bitmap(Gc<'gc, BitmapCharacter<'gc>>),
    Avm1Button(Avm1Button<'gc>),
    Avm2Button(Avm2Button<'gc>),
    Font(Font<'gc>),
    MorphShape(MorphShape<'gc>),
    Text(Text<'gc>),
    Sound(#[collect(require_static)] SoundHandle),
    Video(Video<'gc>),
    BinaryData(Gc<'gc, BinaryData>),
}

#[derive(Collect, Debug)]
#[collect(no_drop)]
pub struct BitmapCharacter<'gc> {
    #[collect(require_static)]
    compressed: CompressedBitmap,
    /// A lazily constructed GPU handle, used when performing fills with this bitmap
    ///
    /// DEIXOU DE SER PRA SEMPRE. Agora pode voltar a ser vazio: a varredura
    /// solta a textura de quem ninguem desenha ha um tempo, e o proximo
    /// desenho a recria a partir de `compressed`, pelo mesmo caminho que a
    /// criou da primeira vez.
    #[collect(require_static)]
    handle: RefCell<Option<BitmapHandle>>,
    /// Quando esta imagem foi desenhada pela ultima vez, no relogio acima.
    #[collect(require_static)]
    usado_em: Cell<u64>,
    /// The bitmap class set by `SymbolClass` - this is used when we instantaite
    /// a `Bitmap` displayobject.
    avm2_class: Lock<BitmapClass<'gc>>,
}

impl<'gc> BitmapCharacter<'gc> {
    pub fn new(compressed: CompressedBitmap) -> Self {
        Self {
            compressed,
            handle: RefCell::new(None),
            usado_em: Cell::new(agora()),
            avm2_class: Lock::new(BitmapClass::NoSubclass),
        }
    }

    pub fn compressed(&self) -> &CompressedBitmap {
        &self.compressed
    }

    pub fn avm2_class(&self) -> BitmapClass<'gc> {
        self.avm2_class.get()
    }

    pub fn set_avm2_class(this: Gc<'gc, Self>, bitmap_class: BitmapClass<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, this), Self, avm2_class).set(bitmap_class);
    }

    pub fn bitmap_handle(
        &self,
        backend: &mut dyn RenderBackend,
    ) -> Result<BitmapHandle, RenderError> {
        // Carimba antes de qualquer coisa: quem pediu o desenho acabou de usar
        // esta imagem, tenha ela textura ou nao.
        self.usado_em.set(agora());

        if let Some(handle) = self.handle.borrow().as_ref() {
            return Ok(handle.clone());
        }
        let decoded = self.compressed.decode()?;
        let new_handle = backend.register_bitmap(decoded)?;
        *self.handle.borrow_mut() = Some(new_handle.clone());
        Ok(new_handle)
    }

    /// Quantos bytes a textura desta imagem ocupa, se ela estiver viva.
    pub fn textura_viva(&self) -> Option<usize> {
        self.handle.borrow().as_ref().map(|_| self.peso_da_textura())
    }

    /// Descomprimida, toda imagem ocupa quatro bytes por ponto.
    fn peso_da_textura(&self) -> usize {
        let tamanho = self.compressed.size();
        tamanho.width as usize * tamanho.height as usize * 4
    }

    /// Solta a textura se ninguem desenhou esta imagem nos ultimos `prazo`
    /// segundos. Devolve quantos bytes sairam — zero se nao soltou nada.
    ///
    /// Soltar aqui nao apaga nada de imediato: quem ja pegou a textura e ainda
    /// a segura continua com ela viva. O que acaba e esta biblioteca ser dona
    /// pra sempre de tudo o que o jogo mostrou uma vez.
    pub fn soltar_se_parada(&self, prazo: u64) -> usize {
        if self.handle.borrow().is_none() {
            return 0;
        }
        if agora().saturating_sub(self.usado_em.get()) < prazo {
            return 0;
        }
        let peso = self.peso_da_textura();
        *self.handle.borrow_mut() = None;
        peso
    }
}

/// Holds a bitmap from an SWF tag, plus the decoded width/height.
/// We avoid decompressing the image until it's actually needed - some pathological SWFS
/// like 'House' have thousands of highly-compressed (mostly empty) bitmaps, which can
/// take over 10GB of ram if we decompress them all during preloading.
#[derive(Clone, Debug)]
pub enum CompressedBitmap {
    Jpeg {
        data: Vec<u8>,
        alpha: Option<Vec<u8>>,
        width: u32,
        height: u32,
    },
    Lossless(DefineBitsLossless<'static>),
}

impl CompressedBitmap {
    pub fn size(&self) -> BitmapSize {
        match self {
            CompressedBitmap::Jpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::Lossless(define_bits_lossless) => BitmapSize {
                width: define_bits_lossless.width.into(),
                height: define_bits_lossless.height.into(),
            },
        }
    }
    pub fn decode(&self) -> Result<RenderBitmap<'static>, RenderError> {
        match self {
            CompressedBitmap::Jpeg {
                data,
                alpha,
                width: _,
                height: _,
            } => ruffle_render::utils::decode_define_bits_jpeg(data, alpha.as_deref()),
            CompressedBitmap::Lossless(define_bits_lossless) => {
                ruffle_render::utils::decode_define_bits_lossless(define_bits_lossless)
            }
        }
    }
}
