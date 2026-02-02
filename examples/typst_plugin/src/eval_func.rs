use fasteval::Compiler;
use fasteval::Evaler;
use num_integer::Integer;
use num_integer::binomial;

pub fn str2compiled(
    expr_str: &str,
) -> Result<(fasteval::Slab, fasteval::Instruction), fasteval::Error> {
    let mut slab: fasteval::Slab = fasteval::Slab::new();
    let compiled: fasteval::Instruction = fasteval::Parser::new()
        .parse(expr_str, &mut slab.ps)?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);

    Ok((slab, compiled))
}

pub fn compiled2func(
    slab: fasteval::Slab,
    compiled: fasteval::Instruction,
) -> impl Fn(f64, f64) -> f64 {
    move |x: f64, y: f64| -> f64 {
        if let fasteval::IConst(c) = compiled {
            c
        } else {
            compiled
                .eval(&slab, &mut |name: &str, args| {
                    calc(name, args).or(match name {
                        "x" => Some(x),
                        "y" => Some(y),
                        _ => None,
                    })
                })
                .unwrap()
        }
    }
}

fn calc(name: &str, args: Vec<f64>) -> Option<f64> {
    match name {
        "pow" => {
            if args.len() == 2 {
                Some(args[0].powf(args[1]))
            } else {
                None
            }
        }
        "exp" => {
            if args.len() == 1 {
                Some(args[0].exp())
            } else {
                None
            }
        }
        "sqrt" => {
            if args.len() == 1 {
                Some(args[0].sqrt())
            } else {
                None
            }
        }
        "root" => {
            if args.len() == 2 {
                Some(args[0].powf(1.0 / args[1]))
            } else {
                None
            }
        }
        "atan2" => {
            if args.len() == 2 {
                Some(args[0].atan2(args[1]))
            } else {
                None
            }
        }
        "ln" => {
            if args.len() == 1 {
                Some(args[0].ln())
            } else {
                None
            }
        }
        "fact" => {
            if args.len() == 1 {
                let n = args[0] as u64;
                let mut result = 1u64;
                for i in 1..=n {
                    result = result.saturating_mul(i);
                }
                Some(result as f64)
            } else {
                None
            }
        }
        "perm" => {
            if args.len() == 2 {
                let n = args[0] as u64;
                let r = args[1] as u64;
                if r > n {
                    return Some(0.0);
                }
                let mut result = 1u64;
                for i in 0..r {
                    result = result.saturating_mul(n - i);
                }
                Some(result as f64)
            } else {
                None
            }
        }
        "binom" => {
            if args.len() == 2 {
                let n = args[0] as u64;
                let k = args[1] as u64;
                Some(binomial(n, k) as f64)
            } else {
                None
            }
        }
        "gcd" => {
            if args.len() == 2 {
                let a = args[0] as u64;
                let b = args[1] as u64;
                Some(a.gcd(&b) as f64)
            } else {
                None
            }
        }
        "lcm" => {
            if args.len() == 2 {
                let a = args[0] as u64;
                let b = args[1] as u64;
                Some(a.lcm(&b) as f64)
            } else {
                None
            }
        }
        "trunc" => {
            if args.len() == 1 {
                Some(args[0].trunc())
            } else {
                None
            }
        }
        "fract" => {
            if args.len() == 1 {
                Some(args[0].fract())
            } else {
                None
            }
        }
        "clamp" => {
            if args.len() == 3 {
                Some(args[0].clamp(args[1], args[2]))
            } else {
                None
            }
        }
        "even" => {
            if args.len() == 1 {
                Some(if (args[0] as i64) % 2 == 0 { 1.0 } else { 0.0 })
            } else {
                None
            }
        }
        "odd" => {
            if args.len() == 1 {
                Some(if (args[0] as i64) % 2 != 0 { 1.0 } else { 0.0 })
            } else {
                None
            }
        }
        "rem" => {
            if args.len() == 2 {
                Some(args[0] % args[1])
            } else {
                None
            }
        }
        "div_euclid" => {
            if args.len() == 2 {
                Some(args[0].div_euclid(args[1]))
            } else {
                None
            }
        }
        "rem_euclid" => {
            if args.len() == 2 {
                Some(args[0].rem_euclid(args[1]))
            } else {
                None
            }
        }
        "quo" => {
            if args.len() == 2 {
                Some((args[0] / args[1]).trunc())
            } else {
                None
            }
        }
        "norm" => {
            if args.len() == 0 {
                None
            } else {
                Some(args.into_iter().map(|x| x * x).sum::<f64>().sqrt())
            }
        }
        "inf" => Some(std::f64::INFINITY),
        "pi" => Some(std::f64::consts::PI),
        "tau" => Some(std::f64::consts::TAU),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}
