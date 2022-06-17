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

impl<A: Aleo, Private: Visibility<A>> Value<A, Private> {
    /// Returns the number of field elements to encode `self`.
    pub(crate) fn num_randomizers(&self) -> u16 {
        match self {
            // Constant and public values do not need to be encrypted.
            Self::Constant(..) | Self::Public(..) => 0u16,
            // Private values need one randomizer per field element.
            Self::Private(private) => private.size_in_fields(),
            // Record values can recursively determine the number of randomizers.
            Self::Record(record) => record.num_randomizers(),
        }
    }
}
