// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use snarkvm_circuit::FromStr;
use snarkvm_compiler::Program;
use snarkvm_utilities::{FromBytes, ToBytes};

fn main() -> anyhow::Result<()> {
    use snarkvm_console::network::Testnet3;

    type CurrentNetwork = Testnet3;
    let program_string = r"program test.aleo;

    function hello:
        input r0 as u16.public;
        input r1 as u16.private;
        input r2 as u32.private;
        input r3 as u32.private;
        xor r0 r1 into r4;
        add r3 r2 into r5;
        output r4 as u16.private;
        output r5 as u32.private;    
    ";
    // Parse a new program.
    let expected = Program::<CurrentNetwork>::from_str(program_string)?;

    // Serialize
    let expected_bytes = expected.to_bytes_le()?;
    //let expected_bytes_with_size_encoding = bincode::serialize(&expected)?;
    //assert_eq!(&expected_bytes[..], &expected_bytes_with_size_encoding[8..]);

    // Deserialize
    println!("TEST PARSE");
    assert_eq!(expected, Program::read_le(&expected_bytes[..])?);
    //assert_eq!(expected, bincode::deserialize(&expected_bytes_with_size_encoding[..])?);

    Ok(())
}
