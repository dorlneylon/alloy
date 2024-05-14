use serde::{Deserialize, Serialize};

/// Internal struct to calculate reward percentiles
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxGasAndReward {
    /// Gas used by the transaction
    pub gas_used: u64,
    /// The effective gas tip by the transaction
    pub reward: u128,
}

impl PartialOrd for TxGasAndReward {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxGasAndReward {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // compare only the reward
        // see:
        // <https://github.com/ethereum/go-ethereum/blob/ee8e83fa5f6cb261dad2ed0a7bbcde4930c41e6c/eth/gasprice/feehistory.go#L85>
        self.reward.cmp(&other.reward)
    }
}

/// Response type for `eth_feeHistory`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    /// An array of block base fees per gas.
    /// This includes the next block after the newest of the returned range,
    /// because this value can be derived from the newest block. Zeroes are
    /// returned for pre-EIP-1559 blocks.
    ///
    /// # Note
    ///
    /// Empty list is skipped only for compatibility with Erigon and Geth.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "alloy_serde::num::u128_vec_via_ruint"
    )]
    pub base_fee_per_gas: Vec<u128>,
    /// An array of block gas used ratios. These are calculated as the ratio
    /// of `gasUsed` and `gasLimit`.
    ///
    /// # Note
    ///
    /// The `Option` is only for compatibility with Erigon and Geth.
    // #[serde(default, with = "alloy_serde::num::f64_hex_vec")]
    pub gas_used_ratio: Vec<f64>,
    /// An array of block base fees per blob gas. This includes the next block after the newest
    /// of  the returned range, because this value can be derived from the newest block. Zeroes
    /// are returned for pre-EIP-4844 blocks.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "alloy_serde::num::u128_vec_via_ruint"
    )]
    pub base_fee_per_blob_gas: Vec<u128>,
    /// An array of block blob gas used ratios. These are calculated as the ratio of gasUsed and
    /// gasLimit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blob_gas_used_ratio: Vec<f64>,
    /// Lowest number block of the returned range.
    #[serde(default, with = "alloy_serde::num::u64_via_ruint")]
    pub oldest_block: u64,
    /// An (optional) array of effective priority fee per gas data points from a single
    /// block. All zeroes are returned if the block is empty.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::num::u128_vec_vec_opt_via_ruint"
    )]
    pub reward: Option<Vec<Vec<u128>>>,
}

impl FeeHistory {
    /// Returns the base fee of the latest block in the `eth_feeHistory` request.
    pub fn latest_block_base_fee(&self) -> Option<u128> {
        // the base fee of requested block is the second last element in the
        // list
        self.base_fee_per_gas.iter().rev().nth(1).copied()
    }

    /// Returns the base fee of the next block.
    pub fn next_block_base_fee(&self) -> Option<u128> {
        self.base_fee_per_gas.last().copied()
    }

    /// Returns the blob base fee of the next block.
    ///
    /// If the next block is pre- EIP-4844, this will return `None`.
    pub fn next_block_blob_base_fee(&self) -> Option<u128> {
        self.base_fee_per_blob_gas
            .last()
            .filter(|fee| {
                // skip zero value that is returned for pre-EIP-4844 blocks
                **fee != 0
            })
            .copied()
    }

    /// Returns the blob fee of the latest block in the `eth_feeHistory` request.
    pub fn latest_block_blob_base_fee(&self) -> Option<u128> {
        // the blob fee requested block is the second last element in the list
        self.base_fee_per_blob_gas
            .iter()
            .rev()
            .nth(1)
            .filter(|fee| {
                // skip zero value that is returned for pre-EIP-4844 blocks
                **fee != 0
            })
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::FeeHistory;

    #[test]
    fn test_fee_history_serde() {
        let sample = r#"{"baseFeePerGas":["0x342770c0","0x2da282a8"],"gasUsedRatio":[0.0],"baseFeePerBlobGas":["0x0","0x0"],"blobGasUsedRatio":[0.0],"oldestBlock":"0x1"}"#;
        let fee_history: FeeHistory = serde_json::from_str(sample).unwrap();
        let expected = FeeHistory {
            base_fee_per_blob_gas: vec![0, 0],
            base_fee_per_gas: vec![875000000, 765625000],
            blob_gas_used_ratio: vec![0.0],
            gas_used_ratio: vec![0.0],
            oldest_block: 1,
            reward: None,
        };

        assert_eq!(fee_history, expected);
        assert_eq!(serde_json::to_string(&fee_history).unwrap(), sample);
    }

    #[test]
    fn test_fee_history_serde_2() {
        let json = r#"{"baseFeePerBlobGas":["0xc0","0xb2","0xab","0x98","0x9e","0x92","0xa4","0xb9","0xd0","0xea","0xfd"],"baseFeePerGas":["0x4cb8cf181","0x53075988e","0x4fb92ee18","0x45c209055","0x4e790dca2","0x58462e84e","0x5b7659f4e","0x5d66ea3aa","0x6283c6e45","0x5ecf0e1e5","0x5da59cf89"],"blobGasUsedRatio":[0.16666666666666666,0.3333333333333333,0,0.6666666666666666,0.16666666666666666,1,1,1,1,0.8333333333333334],"gasUsedRatio":[0.8288135,0.3407616666666667,0,0.9997232,0.999601,0.6444664333333333,0.5848306333333333,0.7189564,0.34952733333333336,0.4509799666666667],"oldestBlock":"0x59f94f","reward":[["0x59682f00"],["0x59682f00"],["0x0"],["0x59682f00"],["0x59682f00"],["0x3b9aca00"],["0x59682f00"],["0x59682f00"],["0x3b9aca00"],["0x59682f00"]]}"#;
        let _actual = serde_json::from_str::<FeeHistory>(json).unwrap();
    }
}
