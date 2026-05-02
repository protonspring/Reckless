use crate::{thread::ThreadData, types::Score};

pub fn correct_eval(td: &ThreadData, raw_eval: i32, correction_value: i32) -> i32 {
    let mut eval = (raw_eval * (20664 + td.board.material())
        + td.optimism[td.board.side_to_move()] * (1487 + td.board.material()))
        / 26685;

    let fmr = td.board.halfmove_clock();
    let fmr_scale = 1.0 as f32 - ((fmr as f32 * fmr as f32) as f32) / 10000.0;
    eval = ((eval as f32) * fmr_scale as f32) as i32;

    eval += correction_value;

    eval.clamp(-Score::TB_WIN_IN_MAX + 1, Score::TB_WIN_IN_MAX - 1)
}
