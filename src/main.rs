use z3::{ast::{Ast, Bool}, Config, Context, SatResult, Solver};

fn main() {
    let mut args = std::env::args();
    let (Some(width), Some(height), Some(puzzle)) = (args.nth(1), args.next(), args.next()) else {
        eprintln!("Usage: ./binairo-plus <width> <height> <puzzle>");
        return;
    };
    let Some((cells, constraints)) = puzzle.split_once('|') else {
        eprintln!("Bad puzzle (no | in descriptor)");
        return;
    };
    let width = width.parse().unwrap();
    let height = height.parse().unwrap();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut vars = Vec::with_capacity(width * height);
    for ch in cells.chars() {
        if ch == '0' {
            vars.push(Bool::from_bool(&ctx, false));
        } else if ch == '1' {
            vars.push(Bool::from_bool(&ctx, true));
        } else {
            for _ in 0..(1 + (ch as usize) - ('a' as usize)) {
                vars.push(Bool::fresh_const(&ctx, ""));
            }
        }
    }
    let solver = Solver::new(&ctx);
    let mut idx = 0;
    for ch in constraints.chars() {
        match ch {
            '1' /* h=    */ => solver.assert(&vars[idx]._eq(&vars[idx + 1])),
            '2' /* hx    */ => solver.assert(&vars[idx].not()._eq(&vars[idx + 1])),
            '3' /* v=    */ => solver.assert(&vars[idx]._eq(&vars[idx + width])),
            '4' /* h=,v= */ => {
                solver.assert(&vars[idx]._eq(&vars[idx + 1]));
                solver.assert(&vars[idx]._eq(&vars[idx + width]));
            }
            '5' /* hx,v= */ => {
                solver.assert(&vars[idx].not()._eq(&vars[idx + 1]));
                solver.assert(&vars[idx]._eq(&vars[idx + width]));
            }
            '6' /* vx    */ => solver.assert(&vars[idx].not()._eq(&vars[idx + width])),
            '7' /* h=,vx */ => {
                solver.assert(&vars[idx]._eq(&vars[idx + 1]));
                solver.assert(&vars[idx].not()._eq(&vars[idx + width]));
            }
            '8' /* hx,vx */ => {
                solver.assert(&vars[idx].not()._eq(&vars[idx + 1]));
                solver.assert(&vars[idx].not()._eq(&vars[idx + width]));
            }
            _ => idx += (ch as usize) - ('a' as usize),
        }
        idx += 1;
    }
    for i in 0..height {
        for j in 0..width {
            if i >= 2 {
                let var1 = &vars[width * i + j];
                let var2 = &vars[width * (i - 1) + j];
                let var3 = &vars[width * (i - 2) + j];
                solver.assert(&Bool::and(&ctx, &[var1, var2, var3]).not());
                solver.assert(&Bool::or(&ctx, &[var1, var2, var3]));
            }
            if j >= 2 {
                let var1 = &vars[width * i + j];
                let var2 = &vars[width * i + j - 1];
                let var3 = &vars[width * i + j - 2];
                solver.assert(&Bool::and(&ctx, &[var1, var2, var3]).not());
                solver.assert(&Bool::or(&ctx, &[var1, var2, var3]));
            }
        }
    }
    for i in 0..height {
        let vars = (0..width).map(|j| (&vars[i * width + j], 1)).collect::<Vec<_>>();
        solver.assert(&Bool::pb_eq(&ctx, &vars, width as i32 / 2));
    }
    for j in 0..width {
        let vars = (0..height).map(|i| (&vars[i * width + j], 1)).collect::<Vec<_>>();
        solver.assert(&Bool::pb_eq(&ctx, &vars, height as i32 / 2));
    }
    assert_eq!(solver.check(), SatResult::Sat);
    let model = solver.get_model().unwrap();
    for i in 0..height {
        for j in 0..width {
            let cell = model.eval(&vars[i * width + j], false).unwrap().as_bool().unwrap();
            if cell {
                print!("⚫");
            } else {
                print!("⚪");
            }
        }
        println!();
    }
}
