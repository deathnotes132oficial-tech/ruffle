//! Special handling for AVM2 orphan objects

use crate::context::UpdateContext;
use crate::display_object::{
    DisplayObject, DisplayObjectWeak, TDisplayObject, TDisplayObjectContainer,
};
use gc_arena::{Collect, Mutation};
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    /// Quantas salas descarregadas ja foram encerradas de verdade.
    static ENCERRADOS: Cell<u64> = const { Cell::new(0) };
    /// Quantos orfaos venceram o prazo e deixaram de receber quadros.
    static ESQUECIDOS: Cell<u64> = const { Cell::new(0) };
}

/// Quantos conteudos descarregados ja foram encerrados.
pub fn contagem_de_encerrados() -> (u64, u64) {
    (
        ENCERRADOS.with(|c| c.get()),
        ESQUECIDOS.with(|c| c.get()),
    )
}

/// The list of 'orphan' objects - these objects have no parent,
/// so we need to manually run their frames in `run_all_phases_avm2` to match
/// Flash's behavior. Clips are added to this list with `add_orphan_movie`.
/// and are removed automatically by `cleanup_dead_orphans`.
///
/// We store `DisplayObjectWeak`, since we don't want to keep these objects
/// alive if they would otherwise be garbage-collected. The movie will
/// stop ticking whenever garbage collection runs if there are no more
/// strong references around (this matches Flash's behavior).
#[derive(Collect)]
#[collect(no_drop)]
pub struct OrphanManager<'gc> {
    /// Cada orfao, com o instante em que ficou orfao.
    ///
    /// O instante existe por causa do prazo la embaixo: sem ele nao da pra
    /// saber ha quanto tempo o clipe esta fora da tela.
    orphans: Rc<Vec<(DisplayObjectWeak<'gc>, u64)>>,

    /// A FILA DE DESCARTE ADIADO.
    ///
    /// O jogo chama `Loader.unload()` ao sair de uma sala. No Flash de
    /// verdade isso NAO encerra o conteudo — so tira a referencia, e a sala
    /// segue rodando ENTER_FRAME, temporizador e som ate o coletor leva-la.
    /// Aqui o coletor nunca leva: o filme fica preso num ciclo de
    /// referencias, entao a sala roda pra sempre. Tres salas criadas viram
    /// tres salas rodando ao mesmo tempo.
    ///
    /// POR QUE ADIADO E NAO NA HORA: encerrar dentro do mesmo quadro quebra,
    /// porque ainda ha referencias vivas em uso naquele instante. Foi o que
    /// derrubou a tentativa anterior de soltar no `Loader.unload()`. Esperar
    /// alguns quadros resolve, e foi assim que resolveram no unico caso
    /// publico de jogo grande rodando por Ruffle em navegador de celular.
    descarte: Rc<Vec<(DisplayObjectWeak<'gc>, u8)>>,
}

/// Quantos quadros esperar antes de encerrar o que foi descarregado.
const ESPERA_DO_DESCARTE: u8 = 5;

/// Quantos segundos fora da tela antes de um orfao parar de receber quadros.
///
/// POR QUE ISTO PRECISA EXISTIR. O Flash deixa o orfao rodando de proposito, e
/// o Ruffle copia esse comportamento — a diferenca e que no Flash o coletor
/// acaba levando o clipe embora, e aqui ele nunca leva, porque o filme esta
/// preso num ciclo de referencias. Entao "roda ate ser coletado" virou "roda
/// pra sempre": medimos 4.607 orfaos vivos numa sessao, todos executando
/// ActionScript a cada quadro. Sao eles que fabricam os milhoes de strings e
/// de nos de XML.
///
/// O prazo faz o papel que o coletor deveria fazer. E generoso de proposito: o
/// jogo pode guardar um clipe fora da tela e devolve-lo depois, e meio minuto
/// e muito mais do que qualquer troca de tela leva.
///
/// E ele so tira os QUADROS, nao destroi nada. Se o jogo devolver o clipe pra
/// arvore, ele volta a rodar normalmente por ela.
const PRAZO_DO_ORFAO: u64 = 30;

impl<'gc> OrphanManager<'gc> {
    fn orphans_mut(&mut self) -> &mut Vec<(DisplayObjectWeak<'gc>, u64)> {
        Rc::make_mut(&mut self.orphans)
    }

    fn descarte_mut(&mut self) -> &mut Vec<(DisplayObjectWeak<'gc>, u8)> {
        Rc::make_mut(&mut self.descarte)
    }

    /// Marca um conteudo descarregado pra ser encerrado daqui a alguns quadros.
    pub fn agendar_descarte(&mut self, dobj: DisplayObject<'gc>) {
        self.descarte_mut()
            .push((dobj.downgrade(), ESPERA_DO_DESCARTE));
    }

    /// Tira um objeto da lista de orfaos — ele para de receber quadros.
    pub fn remover_orfao(&mut self, dobj: DisplayObject<'gc>) {
        self.orphans_mut()
            .retain(|(d, _)| !std::ptr::eq(d.as_ptr(), dobj.as_ptr()));
    }

    /// Encerra o que ja esperou tempo suficiente.
    ///
    /// Chamado uma vez por quadro. Quem ainda nao venceu o prazo so tem o
    /// contador decrementado; quem venceu tem a linha do tempo parada, os
    /// filhos parados junto, e sai da lista de orfaos — que e o que faz a
    /// sala deixar de consumir um quadro atras do outro.
    ///
    /// Quem morreu sozinho no meio do caminho simplesmente some da fila: o
    /// coletor fez o trabalho e nao ha o que encerrar.
    pub fn processar_descarte(context: &mut UpdateContext<'gc>) {
        if context.orphan_manager.descarte.is_empty() {
            return;
        }

        let fila: Rc<_> = context.orphan_manager.descarte.clone();
        let mut restantes: Vec<(DisplayObjectWeak<'gc>, u8)> = Vec::new();
        let mut vencidos: Vec<DisplayObject<'gc>> = Vec::new();

        for (fraco, falta) in fila.iter() {
            if let Some(dobj) = valid_orphan(*fraco, context.gc()) {
                if *falta == 0 {
                    vencidos.push(dobj);
                } else {
                    restantes.push((*fraco, falta - 1));
                }
            }
        }

        *context.orphan_manager.descarte_mut() = restantes;

        for dobj in vencidos {
            Self::encerrar(dobj, context);
            context.orphan_manager.remover_orfao(dobj);
            ENCERRADOS.with(|c| c.set(c.get() + 1));
        }
    }

    /// Para a linha do tempo deste objeto e de tudo que esta dentro dele.
    fn encerrar(dobj: DisplayObject<'gc>, context: &mut UpdateContext<'gc>) {
        if let Some(clip) = dobj.as_movie_clip() {
            clip.stop(context);
        }

        if let Some(container) = dobj.as_container() {
            let filhos: Vec<_> = container.iter_render_list().collect();
            for filho in filhos {
                Self::encerrar(filho, context);
            }
        }
    }

    /// Adds a `MovieClip` to the orphan list. In AVM2, movies advance their
    /// frames even when they are not on a display list. Unfortunately,
    /// multiple SWFS rely on this behavior, so we need to match Flash's
    /// behavior. This should not be called manually - `movie_clip` will
    /// call it when necessary.
    pub fn add_orphan_obj(&mut self, dobj: DisplayObject<'gc>) {
        // Note: comparing pointers is correct because GcWeak keeps its allocation alive,
        // so the pointers can't overlap by accident.
        if self
            .orphans
            .iter()
            .all(|(d, _)| !std::ptr::eq(d.as_ptr(), dobj.as_ptr()))
        {
            let agora = crate::character::agora();
            self.orphans_mut().push((dobj.downgrade(), agora));
        }
    }

    /// SONDA — quantos orfaos estao na lista, e quantos deles ainda vivos.
    ///
    /// Um clipe que sai da tela mas continua vivo fica aqui rodando os quadros
    /// dele PARA SEMPRE. A lista guarda referencia fraca de proposito, entao
    /// ele deveria sumir quando o coletor o levasse — mas os filmes das salas
    /// nunca morrem (medido: todos presos pelo dominio), e um filme vivo
    /// segura os clipes dele.
    ///
    /// Se este numero sobe a cada sala criada e nunca desce, e isto que faz a
    /// memoria subir com o jogo parado: as salas que voce ja deixou continuam
    /// rodando, todas juntas.
    pub fn quantos(&self, mc: &Mutation<'gc>) -> (usize, usize) {
        let total = self.orphans.len();
        let vivos = self
            .orphans
            .iter()
            .filter(|(d, _)| valid_orphan(*d, mc).is_some())
            .count();
        (total, vivos)
    }

    pub fn each_orphan_obj(
        context: &mut UpdateContext<'gc>,
        mut f: impl FnMut(DisplayObject<'gc>, &mut UpdateContext<'gc>),
    ) {
        // Clone the Rc before iterating over it. Any modifications must go through
        // `Rc::make_mut` in `orphan_objects_mut`, which will leave this `Rc` unmodified.
        // This ensures that any orphan additions/removals done by `f` will not affect
        // the iteration in this method.
        let orphan_objs: Rc<_> = context.orphan_manager.orphans.clone();

        for (orphan, _) in orphan_objs.iter() {
            if let Some(dobj) = valid_orphan(*orphan, context.gc()) {
                f(dobj, context);
            }
        }
    }

    /// Called at the end of `run_all_phases_avm2` - removes any movies
    /// that have been garbage collected, or are no longer orphans
    /// (they've since acquired a parent).
    pub fn cleanup_dead_orphans(&mut self, mc: &Mutation<'gc>) {
        let agora = crate::character::agora();

        self.orphans_mut().retain(|(d, desde)| {
            if let Some(dobj) = valid_orphan(*d, mc) {
                // VENCEU O PRAZO: sai da lista e para de receber quadros.
                //
                // Ver `PRAZO_DO_ORFAO`. Nada e destruido — se o jogo devolver
                // o clipe pra arvore, ele volta a rodar por ela.
                if agora.saturating_sub(*desde) >= PRAZO_DO_ORFAO {
                    ESQUECIDOS.with(|c| c.set(c.get() + 1));
                    return false;
                }
                // All clips that become orphaned (have their parent removed, or start out with no parent)
                // get added to the orphan list. However, there's a distinction between clips
                // that are removed from a RemoveObject tag, and clips that are removed from ActionScript.
                //
                // Clips removed from a RemoveObject tag only stay on the orphan list until the end
                // of the frame - this lets them run a framescript (with 'this.parent == null')
                // before they're removed. After that, they're removed from the orphan list,
                // and will not be run in any way.
                //
                // Clips removed from ActionScript stay on the orphan list, and will be run
                // indefinitely (if there are no remaining strong references, they will eventually
                // be garbage collected).
                //
                // To detect this, we check 'placed_by_avm2_script'. This flag get set to 'true'
                // for objects constructed from ActionScript, and for objects moved around
                // in the timeline (add/remove child, swap depths) by ActionScript. A
                // RemoveObject tag will only affect objects instantiated by the timeline,
                // which have not been moved in the displaylist by ActionScript. Therefore,
                // any orphan we see that has 'placed_by_avm2_script()' should stay on the orphan
                // list, because it was not removed by a RemoveObject tag.
                dobj.placed_by_avm2_script()
            } else {
                false
            }
        });
    }
}

impl<'gc> Default for OrphanManager<'gc> {
    fn default() -> Self {
        Self {
            orphans: Rc::new(Vec::new()),
            descarte: Rc::new(Vec::new()),
        }
    }
}

/// If the provided `DisplayObjectWeak` should have frames run, returns
/// Some(clip) with an upgraded `MovieClip`.
/// If this returns `None`, the entry should be removed from the orphan list.
fn valid_orphan<'gc>(
    dobj: DisplayObjectWeak<'gc>,
    mc: &Mutation<'gc>,
) -> Option<DisplayObject<'gc>> {
    dobj.upgrade(mc).filter(|dobj| dobj.parent().is_none())
}
