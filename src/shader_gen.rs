use std::{str::Chars, iter::Peekable};

use crate::settings::{self, Settings};

pub fn assemble_shader(shader: &str, settings: &Settings) -> String {
    
    let mut res = String::from("");
    let mut iter = shader.chars().peekable();
    while iter.peek() != None {
        let char = iter.next().unwrap();
        if char == '$' {
            res.push_str(&handle_conditional_blocks(&mut iter, settings));
        } else {
            res.push(char);
        }
    }
    
    return res;
}

fn handle_conditional_blocks(iter: &mut Peekable<Chars<'_>>, settings: &Settings) -> String {
    let mut token = String::from("");
    let mut tokens = vec![];
    while iter.peek() != None && *iter.peek().unwrap() != '{' {
        let char = iter.next().unwrap();
        if char != ' ' {
            token.push(char);
        } else {
            tokens.push(token);
            token = String::from("");
        }
    }
    iter.next(); // remove {
    let mut res = String::from("");
    let mut open_brackets = 0;
    while iter.peek() != None && (*iter.peek().unwrap() != '}' || open_brackets > 0){
        let char = iter.next().unwrap();
        if char == '$' {
            res.push_str(&handle_conditional_blocks(iter, settings));
        } else {
            if char == '{' {
                open_brackets += 1;
            }
            if char == '}' {
                open_brackets -= 1;
            }
            res.push(char);
        }
    }
    iter.next(); // remove }

    if evaluate_tokens(tokens, settings) {
        return res;
    }

    return String::from("");
}

fn evaluate_tokens(tokens: Vec<String>, settings: &Settings) -> bool {
    let mut res = false;

    for token in tokens {
        match token.as_str() {
            "LIGHTING"         => { res |= settings.view.lighting;            },
            "BONDS"            => { res |= settings.view.render_bonds;        },
            "ROTATION"         => { res |= settings.view.render_rot;          },
            "COLOR-ROTATION"   => { res |= settings.view.color_code_rot;      },
            "SQUARE-PARTICLES" => { res |= !settings.view.circular_particles; },
            "ROUND-PARTICLES"  => { res |= settings.view.circular_particles;  },
            "F32"              => { res |= !settings.simulation.use_f64;      },
            "F64"              => { res |= settings.simulation.use_f64;       },
            "2D"               => { res |= !settings.simulation.D3;           },
            "3D"               => { res |= settings.simulation.D3;            },
            _ => { }
        }
    }
     
    return res;
}