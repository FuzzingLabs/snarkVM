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

use crate::{ProgramID, Register, Value, ValueType};
use snarkvm_console_network::Network;
use snarkvm_console_types::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputID<N: Network> {
    /// The hash of the constant output.
    Constant(Field<N>),
    /// The hash of the public output.
    Public(Field<N>),
    /// The ciphertext hash of the private output.
    Private(Field<N>),
    /// The `(commitment, checksum)` tuple of the record output.
    Record(Field<N>, Field<N>),
    /// The hash of the external record output.
    ExternalRecord(Field<N>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response<N: Network> {
    /// The output ID for the transition.
    output_ids: Vec<OutputID<N>>,
    /// The function outputs.
    outputs: Vec<Value<N>>,
}

impl<N: Network> From<(Vec<OutputID<N>>, Vec<Value<N>>)> for Response<N> {
    /// Note: This method is used to eject from a circuit.
    fn from((output_ids, outputs): (Vec<OutputID<N>>, Vec<Value<N>>)) -> Self {
        Self { output_ids, outputs }
    }
}

impl<N: Network> Response<N> {
    /// Initializes a new response.
    pub fn new(
        program_id: &ProgramID<N>,
        num_inputs: usize,
        tvk: &Field<N>,
        tcm: &Field<N>,
        outputs: Vec<Value<N>>,
        output_types: &[ValueType<N>],
        output_registers: &[Register<N>],
    ) -> Result<Self> {
        // Compute the output IDs.
        let output_ids = outputs
            .iter()
            .zip_eq(output_types)
            .zip_eq(output_registers)
            .enumerate()
            .map(|(index, ((output, output_type), output_register))| {
                match output_type {
                    // For a constant output, compute the hash (using `tcm`) of the output.
                    ValueType::Constant(..) => {
                        // Ensure the output is a plaintext.
                        ensure!(matches!(output, Value::Plaintext(..)), "Expected a plaintext output");

                        // Construct the (console) output index as a field element.
                        let index = Field::from_u16(
                            u16::try_from(num_inputs + index).or_halt_with::<N>("Output index exceeds u16"),
                        );
                        // Construct the preimage as `(output || tcm || index)`.
                        let mut preimage = output.to_fields()?;
                        preimage.push(*tcm);
                        preimage.push(index);
                        // Hash the output to a field element.
                        let output_hash = N::hash_psd8(&preimage)?;

                        // Return the output ID.
                        Ok(OutputID::Constant(output_hash))
                    }
                    // For a public output, compute the hash (using `tcm`) of the output.
                    ValueType::Public(..) => {
                        // Ensure the output is a plaintext.
                        ensure!(matches!(output, Value::Plaintext(..)), "Expected a plaintext output");

                        // Construct the (console) output index as a field element.
                        let index = Field::from_u16(
                            u16::try_from(num_inputs + index).or_halt_with::<N>("Output index exceeds u16"),
                        );
                        // Construct the preimage as `(output || tcm || index)`.
                        let mut preimage = output.to_fields()?;
                        preimage.push(*tcm);
                        preimage.push(index);
                        // Hash the output to a field element.
                        let output_hash = N::hash_psd8(&preimage)?;

                        // Return the output ID.
                        Ok(OutputID::Public(output_hash))
                    }
                    // For a private output, compute the ciphertext (using `tvk`) and hash the ciphertext.
                    ValueType::Private(..) => {
                        // Ensure the output is a plaintext.
                        ensure!(matches!(output, Value::Plaintext(..)), "Expected a plaintext output");
                        // Construct the (console) output index as a field element.
                        let index = Field::from_u16(
                            u16::try_from(num_inputs + index).or_halt_with::<N>("Output index exceeds u16"),
                        );
                        // Compute the output view key as `Hash(tvk || index)`.
                        let output_view_key = N::hash_psd2(&[*tvk, index])?;
                        // Compute the ciphertext.
                        let ciphertext = match &output {
                            Value::Plaintext(plaintext) => plaintext.encrypt_symmetric(output_view_key)?,
                            // Ensure the output is a plaintext.
                            Value::Record(..) => bail!("Expected a plaintext output, found a record output"),
                        };
                        // Hash the ciphertext to a field element.
                        let output_hash = N::hash_psd8(&ciphertext.to_fields()?)?;
                        // Return the output ID.
                        Ok(OutputID::Private(output_hash))
                    }
                    // For a record output, compute the record commitment, and encrypt the record (using `tvk`).
                    ValueType::Record(record_name) => {
                        // Retrieve the record.
                        let record = match &output {
                            Value::Record(record) => record,
                            // Ensure the input is a record.
                            Value::Plaintext(..) => bail!("Expected a record output, found a plaintext output"),
                        };

                        // Compute the record commitment.
                        let commitment = record.to_commitment(program_id, record_name)?;

                        // Construct the (console) output index as a field element.
                        let index = Field::from_u64(output_register.locator());
                        // Compute the encryption randomizer as `HashToScalar(tvk || index)`.
                        let randomizer = N::hash_to_scalar_psd2(&[*tvk, index])?;

                        // Encrypt the record, using the randomizer.
                        let encrypted_record = record.encrypt(randomizer)?;
                        // Compute the record checksum, as the hash of the encrypted record.
                        let checksum = N::hash_bhp1024(&encrypted_record.to_bits_le())?;

                        // Return the output ID.
                        Ok(OutputID::Record(commitment, checksum))
                    }
                    // For a locator output, compute the hash (using `tvk`) of the output.
                    ValueType::ExternalRecord(..) => {
                        // Ensure the output is a record.
                        ensure!(matches!(output, Value::Record(..)), "Expected a record output");

                        // Construct the (console) output index as a field element.
                        let index = Field::from_u16(
                            u16::try_from(num_inputs + index).or_halt_with::<N>("Output index exceeds u16"),
                        );
                        // Construct the preimage as `(output || tvk || index)`.
                        let mut preimage = output.to_fields()?;
                        preimage.push(*tvk);
                        preimage.push(index);
                        // Hash the output to a field element.
                        let output_hash = N::hash_psd8(&preimage)?;

                        // Return the output ID.
                        Ok(OutputID::ExternalRecord(output_hash))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { output_ids, outputs })
    }

    /// Returns the output ID for the transition.
    pub fn output_ids(&self) -> &[OutputID<N>] {
        &self.output_ids
    }

    /// Returns the function outputs.
    pub fn outputs(&self) -> &[Value<N>] {
        &self.outputs
    }
}
