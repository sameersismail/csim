/// Parse Valgrind memory trace outputs
// To generate:
//     valgrind --log-fd=1 --tool=lackey -v --trace-mem=yes <program>

use std::error::Error;

#[derive(PartialEq, Debug)]
pub enum Operation {
    Load,
    Store,
    Modify,
    Instruction, 
}

#[derive(Debug, PartialEq)]
pub struct MemoryAccess {
    pub operation: Operation,
    pub address: u64,
    size: u8,
}

pub fn parse(trace_input: &str) -> Result<Vec<MemoryAccess>, Box<dyn Error>> {
    let mut traces: Vec<MemoryAccess> = vec![];

    for line in trace_input.lines() {
        let trace: Vec<&str> = line.trim().split(" ").filter(|el| !el.is_empty()).collect();

        if trace.len() != 2 {
            return Err("Error: Parsing".into());
        }
        
        let operation = parse_operation(trace[0])?;
        let (address, size) = parse_address_size(trace[1])?;

        if operation == Operation::Modify {
            // A modify is a load and store
            traces.push(MemoryAccess {
                operation: Operation::Load,
                address: address,
                size: size,
            });

            traces.push(MemoryAccess {
                operation: Operation::Store,
                address: address,
                size: size,
            });
        } else if operation == Operation::Instruction {
            // Ignore instruction accesses
        } else {
            traces.push(MemoryAccess {
                operation: operation,
                address: address,
                size: size,
            });
        }
    }
    
    Ok(traces)
}

fn parse_address_size(item: &str) -> Result<(u64, u8), Box<dyn Error>> {
    let operands: Vec<&str> = item.split(",").collect();

    if operands.len() != 2 {
        return Err("Error: Parsing".into());
    }

    let address = u64::from_str_radix(operands[0], 16)?;
    let size = u8::from_str_radix(operands[1], 10)?;

    Ok((address, size))
}

fn parse_operation(op: &str) -> Result<Operation, Box<dyn Error>> {
    match op {
        "L" => Ok(Operation::Load),
        "S" => Ok(Operation::Store),
        "M" => Ok(Operation::Modify),
        "I" => Ok(Operation::Instruction),
        _ => Err("Error: Parsing".into())
    }
}

#[cfg(test)]
mod test {
    use crate::valgrind::{parse, MemoryAccess, Operation};

    #[test]
    fn basic_parsing() {
        let instructions = "\
I 10,1
 L 10,1
 M 20,1
 L 22,1
 S 18,1
 L 110,1
 L 210,1
 M 12,1";

        let result = parse(instructions).unwrap();
        assert_eq!(result,
            vec![
            MemoryAccess {
                operation: Operation::Load,
                address: 0x10,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Load,
                address: 0x20,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Store,
                address: 0x20,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Load,
                address: 0x22,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Store,
                address: 0x18,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Load,
                address: 0x110, 
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Load,
                address: 0x210,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Load,
                address: 0x12,
                size: 1,
            },
            MemoryAccess {
                operation: Operation::Store,
                address: 0x12,
                size: 1,
            },
        ]);
    }
    
    #[test]
    fn noop() {
        let instructions = "I 10,1";
        let result = parse(instructions).unwrap();
        assert_eq!(result, vec![]);
    }
}
