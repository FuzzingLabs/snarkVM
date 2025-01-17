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

use super::*;

static GRAPH_KEY_PREFIX: [u8; 9] = [42, 72, 193, 144, 65, 126, 212, 229, 211]; // AGraphKey1

impl<N: Network> FromStr for GraphKey<N> {
    type Err = Error;

    /// Reads in an account graph key from a base58 string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Encode the string into base58.
        let data = s.from_base58().map_err(|err| anyhow!("{:?}", err))?;
        if data.len() != 41 {
            bail!("Invalid account graph key length: found {}, expected 41", data.len())
        } else if data[0..9] != GRAPH_KEY_PREFIX {
            bail!("Invalid account graph key prefix: found {:?}, expected {:?}", &data[0..9], GRAPH_KEY_PREFIX)
        }
        // Output the graph key.
        Self::try_from(Field::read_le(&data[9..41])?)
    }
}

impl<N: Network> fmt::Display for GraphKey<N> {
    /// Writes the account graph key as a base58 string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write the graph key bytes.
        let mut graph_key = [0u8; 41];
        graph_key[0..9].copy_from_slice(&GRAPH_KEY_PREFIX);
        self.sk_tag.write_le(&mut graph_key[9..41]).map_err(|_| fmt::Error)?;
        // Encode the graph key into base58.
        write!(f, "{}", graph_key.to_base58())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrivateKey;
    use snarkvm_console_network::Testnet3;

    type CurrentNetwork = Testnet3;

    const ITERATIONS: u64 = 10_000;

    #[test]
    fn test_string() -> Result<()> {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Sample a new graph key.
            let private_key = PrivateKey::<CurrentNetwork>::new(&mut rng)?;
            let view_key = ViewKey::try_from(private_key)?;
            let expected = GraphKey::try_from(view_key)?;

            // Check the string representation.
            let candidate = format!("{expected}");
            assert_eq!(expected, GraphKey::from_str(&candidate)?);
            assert_eq!("AGraphKey", candidate.split('1').next().unwrap());
        }
        Ok(())
    }
}
