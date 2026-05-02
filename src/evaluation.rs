use crate::{thread::ThreadData, types::Score};

pub fn correct_eval(td: &ThreadData, raw_eval: i32, correction_value: i32) -> i32 {
    let mut eval = (raw_eval * (20664 + td.board.material())
        + td.optimism[td.board.side_to_move()] * (1487 + td.board.material()))
        / 26685;

 
    //let old_scale = (200 - td.board.halfmove_clock() as i32) / 200;
    let fmr_scale = (200.0 - ((td.board.halfmove_clock() as f32 - 35.0)/ 12.0).exp()) / 200.0;
    //println!("FMR {}, {}", td.board.halfmove_clock(), fmr_scale);
    //println!("fmr: {}, old {}, new: {}", td.board.halfmove_clock(), old_scale, fmr_scale);

    //eval = (eval as f32 * fmr_scale) as i32;

    //eval = eval * old_scale; //(200 - td.board.halfmove_clock() as i32) / 200;

    //eval = eval * (200 - td.board.halfmove_clock() as i32) / 200;
    eval = (eval as f32 * fmr_scale) as i32;
    //println!("fmr: {}, old {}, new scale: {}, eval: {}, new eval: {}", td.board.halfmove_clock(), old_scale, fmr_scale, eval, eval2);
    //println!("fmr: {}, new scale: {}, new eval: {}", td.board.halfmove_clock(), fmr_scale, eval);

    eval += correction_value;

    eval.clamp(-Score::TB_WIN_IN_MAX + 1, Score::TB_WIN_IN_MAX - 1)
}
