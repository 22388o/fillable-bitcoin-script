use bitcoin::opcodes::all::OP_RESERVED;
use bitcoin::script::Instruction;
use bitcoin::Opcode;
use bitvm::treepp::*;

pub const OP_PLACEHOLDER: Opcode = OP_RESERVED;

pub struct FillableScript {
    pub scripts: Vec<Vec<u8>>,
}

impl From<Script> for FillableScript {
    fn from(script: Script) -> Self {
        let instructions_iter = script.instructions();

        let mut scripts = vec![];
        let mut cur_script = vec![];

        for instruction in instructions_iter {
            let instruction = instruction.expect("error interpreting the script");

            match instruction {
                Instruction::PushBytes(v) => {
                    let len = v.len();
                    if len <= 75 {
                        cur_script.extend_from_slice(&[len as u8]);
                        cur_script.extend_from_slice(v.as_bytes());
                    } else if len <= 255 {
                        cur_script.extend_from_slice(&[0x4c]);
                        cur_script.extend_from_slice(&[len as u8]);
                        cur_script.extend_from_slice(v.as_bytes());
                    } else if len <= 65535 {
                        cur_script.extend_from_slice(&[0x4d]);
                        cur_script
                            .extend_from_slice(&[(len & 0xff) as u8, ((len >> 8) & 0xff) as u8]);
                        cur_script.extend_from_slice(v.as_bytes());
                    } else {
                        cur_script.extend_from_slice(&[0x4e]);
                        cur_script.extend_from_slice(&[
                            (len & 0xff) as u8,
                            ((len >> 8) & 0xff) as u8,
                            ((len >> 16) & 0xff) as u8,
                            ((len >> 24) & 0xff) as u8,
                        ]);
                        cur_script.extend_from_slice(v.as_bytes());
                    }
                }
                Instruction::Op(opcode) => {
                    if opcode != OP_RESERVED {
                        cur_script.extend_from_slice(&[opcode.to_u8()]);
                    } else {
                        scripts.push(cur_script.clone());
                        cur_script.clear();
                    }
                }
            }
        }

        if !cur_script.is_empty() {
            scripts.push(cur_script);
        }

        Self { scripts }
    }
}

#[cfg(test)]
mod test {
    use crate::FillableScript;
    use bitvm::treepp::*;

    #[test]
    fn test_fillable_script() {
        let script = script! {
            OP_PUSHBYTES_1 OP_PUSHBYTES_1
            OP_RESERVED
            OP_CAT
            OP_PUSHBYTES_2 OP_PUSHBYTES_1 OP_PUSHBYTES_2
            OP_EQUAL
        };

        let scripts = FillableScript::from(script);
        assert_eq!(
            scripts.scripts[0],
            script! {
                OP_PUSHBYTES_1 OP_PUSHBYTES_1
            }
            .as_bytes()
        );
        assert_eq!(
            scripts.scripts[1],
            script! {
                OP_CAT
                OP_PUSHBYTES_2 OP_PUSHBYTES_1 OP_PUSHBYTES_2
                OP_EQUAL
            }
            .as_bytes()
        );
    }
}
