package {
import flash.display.Sprite;

// Whole numbers that AVM2 keeps as Number rather than int (at or past 2^28, -0, uint, typed Number) are
// written without a fraction, as Flash writes them.
public class Test extends Sprite {
    function Test() {
        var negZero:Number = 0;
        negZero = -negZero;
        var typed:Number = 5;
        var big:uint = 4294967295;

        test(1500000000);
        test(268435456);
        test(-268435457);
        test(4294967296);
        test(9007199254740992);
        test(-9007199254740992);
        test(negZero);
        test(typed);
        test(big);
        test(1790499068112);
        test(0.5);
        test(-2.25);
        test(1.5e-7);
        test(NaN);
        test(Infinity);
        test(-Infinity);
        test([1500000000, negZero, 0.5, 3]);
        test({CharItemID: 1500000000});
    }

    private function test(value: *): void {
        trace(JSON.stringify(value));
    }
}
}
