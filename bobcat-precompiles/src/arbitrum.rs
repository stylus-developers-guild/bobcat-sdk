#![allow(unused, non_snake_case)]

use bobcat_maths::U;

use bobcat_cd::{const_keccak_sel, leftpad_addr, leftpad_bool, leftpad_u16, leftpad_u32, leftpad_u8, leftpad_usize};

use bobcat_call::{call_slice, call_unit, static_call_slice, static_call_unit};

#[cfg(feature = "alloc")]
use bobcat_call::{call_vec, static_call_vec};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use array_concat::concat_arrays;

/// Left-pad a u64 into a 32-byte ABI word.
pub const fn leftpad_u64(x: u64) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 8], x.to_be_bytes())
}

/// Address of the `arb_sys` precompile.
pub const ADDR_ARB_SYS: [u8; 20] = U::from_u32(100).const_addr();
/// Address of the `arb_info` precompile.
pub const ADDR_ARB_INFO: [u8; 20] = U::from_u32(101).const_addr();
/// Address of the `arb_address_table` precompile.
pub const ADDR_ARB_ADDRESS_TABLE: [u8; 20] = U::from_u32(102).const_addr();
/// Address of the `arb_function_table` precompile.
pub const ADDR_ARB_FUNCTION_TABLE: [u8; 20] = U::from_u32(104).const_addr();
/// Address of the `arbos_test` precompile.
pub const ADDR_ARBOS_TEST: [u8; 20] = U::from_u32(105).const_addr();
/// Address of the `arb_owner_public` precompile.
pub const ADDR_ARB_OWNER_PUBLIC: [u8; 20] = U::from_u32(107).const_addr();
/// Address of the `arb_gas_info` precompile.
pub const ADDR_ARB_GAS_INFO: [u8; 20] = U::from_u32(108).const_addr();
/// Address of the `arb_aggregator` precompile.
pub const ADDR_ARB_AGGREGATOR: [u8; 20] = U::from_u32(109).const_addr();
/// Address of the `arb_retryable_tx` precompile.
pub const ADDR_ARB_RETRYABLE_TX: [u8; 20] = U::from_u32(110).const_addr();
/// Address of the `arb_statistics` precompile.
pub const ADDR_ARB_STATISTICS: [u8; 20] = U::from_u32(111).const_addr();
/// Address of the `arb_owner` precompile.
pub const ADDR_ARB_OWNER: [u8; 20] = U::from_u32(112).const_addr();
/// Address of the `arb_wasm` precompile.
pub const ADDR_ARB_WASM: [u8; 20] = U::from_u32(113).const_addr();
/// Address of the `arb_wasm_cache` precompile.
pub const ADDR_ARB_WASM_CACHE: [u8; 20] = U::from_u32(114).const_addr();
/// Address of the `arb_native_token_manager` precompile.
pub const ADDR_ARB_NATIVE_TOKEN_MANAGER: [u8; 20] = U::from_u32(115).const_addr();
/// Address of the `arb_debug` precompile.
pub const ADDR_ARB_DEBUG: [u8; 20] = U::from_u32(255).const_addr();

pub const SEL_ARB_SYS_ARBBLOCKNUMBER: [u8; 4] = const_keccak_sel(b"arbBlockNumber()");
pub const SEL_ARB_SYS_ARBBLOCKHASH: [u8; 4] = const_keccak_sel(b"arbBlockHash(uint256)");
pub const SEL_ARB_SYS_ARBCHAINID: [u8; 4] = const_keccak_sel(b"arbChainID()");
pub const SEL_ARB_SYS_ARBOSVERSION: [u8; 4] = const_keccak_sel(b"arbOSVersion()");
pub const SEL_ARB_SYS_GETSTORAGEGASAVAILABLE: [u8; 4] = const_keccak_sel(b"getStorageGasAvailable()");
pub const SEL_ARB_SYS_ISTOPLEVELCALL: [u8; 4] = const_keccak_sel(b"isTopLevelCall()");
pub const SEL_ARB_SYS_MAPL1SENDERCONTRACTADDRESSTOL2ALIAS: [u8; 4] = const_keccak_sel(b"mapL1SenderContractAddressToL2Alias(address,address)");
pub const SEL_ARB_SYS_WASMYCALLERSADDRESSALIASED: [u8; 4] = const_keccak_sel(b"wasMyCallersAddressAliased()");
pub const SEL_ARB_SYS_MYCALLERSADDRESSWITHOUTALIASING: [u8; 4] = const_keccak_sel(b"myCallersAddressWithoutAliasing()");
pub const SEL_ARB_SYS_WITHDRAWETH: [u8; 4] = const_keccak_sel(b"withdrawEth(address)");
pub const SEL_ARB_SYS_SENDTXTOL1: [u8; 4] = const_keccak_sel(b"sendTxToL1(address,bytes)");
pub const SEL_ARB_SYS_SENDMERKLETREESTATE: [u8; 4] = const_keccak_sel(b"sendMerkleTreeState()");

pub const SEL_ARB_INFO_GETBALANCE: [u8; 4] = const_keccak_sel(b"getBalance(address)");
pub const SEL_ARB_INFO_GETCODE: [u8; 4] = const_keccak_sel(b"getCode(address)");

pub const SEL_ARB_ADDRESS_TABLE_ADDRESSEXISTS: [u8; 4] = const_keccak_sel(b"addressExists(address)");
pub const SEL_ARB_ADDRESS_TABLE_COMPRESS: [u8; 4] = const_keccak_sel(b"compress(address)");
pub const SEL_ARB_ADDRESS_TABLE_DECOMPRESS: [u8; 4] = const_keccak_sel(b"decompress(bytes,uint256)");
pub const SEL_ARB_ADDRESS_TABLE_LOOKUP: [u8; 4] = const_keccak_sel(b"lookup(address)");
pub const SEL_ARB_ADDRESS_TABLE_LOOKUPINDEX: [u8; 4] = const_keccak_sel(b"lookupIndex(uint256)");
pub const SEL_ARB_ADDRESS_TABLE_REGISTER: [u8; 4] = const_keccak_sel(b"register(address)");
pub const SEL_ARB_ADDRESS_TABLE_SIZE: [u8; 4] = const_keccak_sel(b"size()");

pub const SEL_ARB_FUNCTION_TABLE_UPLOAD: [u8; 4] = const_keccak_sel(b"upload(bytes)");
pub const SEL_ARB_FUNCTION_TABLE_SIZE: [u8; 4] = const_keccak_sel(b"size(address)");
pub const SEL_ARB_FUNCTION_TABLE_GET: [u8; 4] = const_keccak_sel(b"get(address,uint256)");

pub const SEL_ARBOS_TEST_BURNARBGAS: [u8; 4] = const_keccak_sel(b"burnArbGas(uint256)");

pub const SEL_ARB_OWNER_PUBLIC_ISCHAINOWNER: [u8; 4] = const_keccak_sel(b"isChainOwner(address)");
pub const SEL_ARB_OWNER_PUBLIC_RECTIFYCHAINOWNER: [u8; 4] = const_keccak_sel(b"rectifyChainOwner(address)");
pub const SEL_ARB_OWNER_PUBLIC_GETALLCHAINOWNERS: [u8; 4] = const_keccak_sel(b"getAllChainOwners()");
pub const SEL_ARB_OWNER_PUBLIC_GETNATIVETOKENMANAGEMENTFROM: [u8; 4] = const_keccak_sel(b"getNativeTokenManagementFrom()");
pub const SEL_ARB_OWNER_PUBLIC_ISNATIVETOKENOWNER: [u8; 4] = const_keccak_sel(b"isNativeTokenOwner(address)");
pub const SEL_ARB_OWNER_PUBLIC_GETALLNATIVETOKENOWNERS: [u8; 4] = const_keccak_sel(b"getAllNativeTokenOwners()");
pub const SEL_ARB_OWNER_PUBLIC_GETTRANSACTIONFILTERINGFROM: [u8; 4] = const_keccak_sel(b"getTransactionFilteringFrom()");
pub const SEL_ARB_OWNER_PUBLIC_ISTRANSACTIONFILTERER: [u8; 4] = const_keccak_sel(b"isTransactionFilterer(address)");
pub const SEL_ARB_OWNER_PUBLIC_GETALLTRANSACTIONFILTERERS: [u8; 4] = const_keccak_sel(b"getAllTransactionFilterers()");
pub const SEL_ARB_OWNER_PUBLIC_GETFILTEREDFUNDSRECIPIENT: [u8; 4] = const_keccak_sel(b"getFilteredFundsRecipient()");
pub const SEL_ARB_OWNER_PUBLIC_GETNETWORKFEEACCOUNT: [u8; 4] = const_keccak_sel(b"getNetworkFeeAccount()");
pub const SEL_ARB_OWNER_PUBLIC_GETINFRAFEEACCOUNT: [u8; 4] = const_keccak_sel(b"getInfraFeeAccount()");
pub const SEL_ARB_OWNER_PUBLIC_GETBROTLICOMPRESSIONLEVEL: [u8; 4] = const_keccak_sel(b"getBrotliCompressionLevel()");
pub const SEL_ARB_OWNER_PUBLIC_GETPARENTGASFLOORPERTOKEN: [u8; 4] = const_keccak_sel(b"getParentGasFloorPerToken()");
pub const SEL_ARB_OWNER_PUBLIC_GETSCHEDULEDUPGRADE: [u8; 4] = const_keccak_sel(b"getScheduledUpgrade()");
pub const SEL_ARB_OWNER_PUBLIC_ISCALLDATAPRICEINCREASEENABLED: [u8; 4] = const_keccak_sel(b"isCalldataPriceIncreaseEnabled()");
pub const SEL_ARB_OWNER_PUBLIC_GETCOLLECTTIPS: [u8; 4] = const_keccak_sel(b"getCollectTips()");
pub const SEL_ARB_OWNER_PUBLIC_GETMAXSTYLUSCONTRACTFRAGMENTS: [u8; 4] = const_keccak_sel(b"getMaxStylusContractFragments()");

pub const SEL_ARB_GAS_INFO_GETPRICESINWEIWITHAGGREGATOR: [u8; 4] = const_keccak_sel(b"getPricesInWeiWithAggregator(address)");
pub const SEL_ARB_GAS_INFO_GETPRICESINWEI: [u8; 4] = const_keccak_sel(b"getPricesInWei()");
pub const SEL_ARB_GAS_INFO_GETPRICESINARBGASWITHAGGREGATOR: [u8; 4] = const_keccak_sel(b"getPricesInArbGasWithAggregator(address)");
pub const SEL_ARB_GAS_INFO_GETPRICESINARBGAS: [u8; 4] = const_keccak_sel(b"getPricesInArbGas()");
pub const SEL_ARB_GAS_INFO_GETGASACCOUNTINGPARAMS: [u8; 4] = const_keccak_sel(b"getGasAccountingParams()");
pub const SEL_ARB_GAS_INFO_GETMAXTXGASLIMIT: [u8; 4] = const_keccak_sel(b"getMaxTxGasLimit()");
pub const SEL_ARB_GAS_INFO_GETMINIMUMGASPRICE: [u8; 4] = const_keccak_sel(b"getMinimumGasPrice()");
pub const SEL_ARB_GAS_INFO_GETL1BASEFEEESTIMATE: [u8; 4] = const_keccak_sel(b"getL1BaseFeeEstimate()");
pub const SEL_ARB_GAS_INFO_GETL1BASEFEEESTIMATEINERTIA: [u8; 4] = const_keccak_sel(b"getL1BaseFeeEstimateInertia()");
pub const SEL_ARB_GAS_INFO_GETL1REWARDRATE: [u8; 4] = const_keccak_sel(b"getL1RewardRate()");
pub const SEL_ARB_GAS_INFO_GETL1REWARDRECIPIENT: [u8; 4] = const_keccak_sel(b"getL1RewardRecipient()");
pub const SEL_ARB_GAS_INFO_GETL1GASPRICEESTIMATE: [u8; 4] = const_keccak_sel(b"getL1GasPriceEstimate()");
pub const SEL_ARB_GAS_INFO_GETCURRENTTXL1GASFEES: [u8; 4] = const_keccak_sel(b"getCurrentTxL1GasFees()");
pub const SEL_ARB_GAS_INFO_GETGASBACKLOG: [u8; 4] = const_keccak_sel(b"getGasBacklog()");
pub const SEL_ARB_GAS_INFO_GETPRICINGINERTIA: [u8; 4] = const_keccak_sel(b"getPricingInertia()");
pub const SEL_ARB_GAS_INFO_GETGASBACKLOGTOLERANCE: [u8; 4] = const_keccak_sel(b"getGasBacklogTolerance()");
pub const SEL_ARB_GAS_INFO_GETL1PRICINGSURPLUS: [u8; 4] = const_keccak_sel(b"getL1PricingSurplus()");
pub const SEL_ARB_GAS_INFO_GETPERBATCHGASCHARGE: [u8; 4] = const_keccak_sel(b"getPerBatchGasCharge()");
pub const SEL_ARB_GAS_INFO_GETAMORTIZEDCOSTCAPBIPS: [u8; 4] = const_keccak_sel(b"getAmortizedCostCapBips()");
pub const SEL_ARB_GAS_INFO_GETL1FEESAVAILABLE: [u8; 4] = const_keccak_sel(b"getL1FeesAvailable()");
pub const SEL_ARB_GAS_INFO_GETL1PRICINGEQUILIBRATIONUNITS: [u8; 4] = const_keccak_sel(b"getL1PricingEquilibrationUnits()");
pub const SEL_ARB_GAS_INFO_GETLASTL1PRICINGUPDATETIME: [u8; 4] = const_keccak_sel(b"getLastL1PricingUpdateTime()");
pub const SEL_ARB_GAS_INFO_GETL1PRICINGFUNDSDUEFORREWARDS: [u8; 4] = const_keccak_sel(b"getL1PricingFundsDueForRewards()");
pub const SEL_ARB_GAS_INFO_GETL1PRICINGUNITSSINCEUPDATE: [u8; 4] = const_keccak_sel(b"getL1PricingUnitsSinceUpdate()");
pub const SEL_ARB_GAS_INFO_GETLASTL1PRICINGSURPLUS: [u8; 4] = const_keccak_sel(b"getLastL1PricingSurplus()");
pub const SEL_ARB_GAS_INFO_GETMAXBLOCKGASLIMIT: [u8; 4] = const_keccak_sel(b"getMaxBlockGasLimit()");
pub const SEL_ARB_GAS_INFO_GETGASPRICINGCONSTRAINTS: [u8; 4] = const_keccak_sel(b"getGasPricingConstraints()");
pub const SEL_ARB_GAS_INFO_GETMULTIGASPRICINGCONSTRAINTS: [u8; 4] = const_keccak_sel(b"getMultiGasPricingConstraints()");
pub const SEL_ARB_GAS_INFO_GETMULTIGASBASEFEE: [u8; 4] = const_keccak_sel(b"getMultiGasBaseFee()");

pub const SEL_ARB_AGGREGATOR_GETPREFERREDAGGREGATOR: [u8; 4] = const_keccak_sel(b"getPreferredAggregator(address)");
pub const SEL_ARB_AGGREGATOR_GETDEFAULTAGGREGATOR: [u8; 4] = const_keccak_sel(b"getDefaultAggregator()");
pub const SEL_ARB_AGGREGATOR_GETBATCHPOSTERS: [u8; 4] = const_keccak_sel(b"getBatchPosters()");
pub const SEL_ARB_AGGREGATOR_ADDBATCHPOSTER: [u8; 4] = const_keccak_sel(b"addBatchPoster(address)");
pub const SEL_ARB_AGGREGATOR_GETFEECOLLECTOR: [u8; 4] = const_keccak_sel(b"getFeeCollector(address)");
pub const SEL_ARB_AGGREGATOR_SETFEECOLLECTOR: [u8; 4] = const_keccak_sel(b"setFeeCollector(address,address)");
pub const SEL_ARB_AGGREGATOR_GETTXBASEFEE: [u8; 4] = const_keccak_sel(b"getTxBaseFee(address)");
pub const SEL_ARB_AGGREGATOR_SETTXBASEFEE: [u8; 4] = const_keccak_sel(b"setTxBaseFee(address,uint256)");

pub const SEL_ARB_RETRYABLE_TX_REDEEM: [u8; 4] = const_keccak_sel(b"redeem(bytes32)");
pub const SEL_ARB_RETRYABLE_TX_GETLIFETIME: [u8; 4] = const_keccak_sel(b"getLifetime()");
pub const SEL_ARB_RETRYABLE_TX_GETTIMEOUT: [u8; 4] = const_keccak_sel(b"getTimeout(bytes32)");
pub const SEL_ARB_RETRYABLE_TX_KEEPALIVE: [u8; 4] = const_keccak_sel(b"keepalive(bytes32)");
pub const SEL_ARB_RETRYABLE_TX_GETBENEFICIARY: [u8; 4] = const_keccak_sel(b"getBeneficiary(bytes32)");
pub const SEL_ARB_RETRYABLE_TX_CANCEL: [u8; 4] = const_keccak_sel(b"cancel(bytes32)");
pub const SEL_ARB_RETRYABLE_TX_GETCURRENTREDEEMER: [u8; 4] = const_keccak_sel(b"getCurrentRedeemer()");
pub const SEL_ARB_RETRYABLE_TX_SUBMITRETRYABLE: [u8; 4] = const_keccak_sel(b"submitRetryable(bytes32,uint256,uint256,uint256,uint256,uint64,uint256,address,address,address,bytes)");

pub const SEL_ARB_STATISTICS_GETSTATS: [u8; 4] = const_keccak_sel(b"getStats()");

pub const SEL_ARB_OWNER_ADDCHAINOWNER: [u8; 4] = const_keccak_sel(b"addChainOwner(address)");
pub const SEL_ARB_OWNER_REMOVECHAINOWNER: [u8; 4] = const_keccak_sel(b"removeChainOwner(address)");
pub const SEL_ARB_OWNER_ISCHAINOWNER: [u8; 4] = const_keccak_sel(b"isChainOwner(address)");
pub const SEL_ARB_OWNER_GETALLCHAINOWNERS: [u8; 4] = const_keccak_sel(b"getAllChainOwners()");
pub const SEL_ARB_OWNER_SETNATIVETOKENMANAGEMENTFROM: [u8; 4] = const_keccak_sel(b"setNativeTokenManagementFrom(uint64)");
pub const SEL_ARB_OWNER_SETTRANSACTIONFILTERINGFROM: [u8; 4] = const_keccak_sel(b"setTransactionFilteringFrom(uint64)");
pub const SEL_ARB_OWNER_ADDNATIVETOKENOWNER: [u8; 4] = const_keccak_sel(b"addNativeTokenOwner(address)");
pub const SEL_ARB_OWNER_REMOVENATIVETOKENOWNER: [u8; 4] = const_keccak_sel(b"removeNativeTokenOwner(address)");
pub const SEL_ARB_OWNER_ISNATIVETOKENOWNER: [u8; 4] = const_keccak_sel(b"isNativeTokenOwner(address)");
pub const SEL_ARB_OWNER_GETALLNATIVETOKENOWNERS: [u8; 4] = const_keccak_sel(b"getAllNativeTokenOwners()");
pub const SEL_ARB_OWNER_ADDTRANSACTIONFILTERER: [u8; 4] = const_keccak_sel(b"addTransactionFilterer(address)");
pub const SEL_ARB_OWNER_REMOVETRANSACTIONFILTERER: [u8; 4] = const_keccak_sel(b"removeTransactionFilterer(address)");
pub const SEL_ARB_OWNER_ISTRANSACTIONFILTERER: [u8; 4] = const_keccak_sel(b"isTransactionFilterer(address)");
pub const SEL_ARB_OWNER_GETALLTRANSACTIONFILTERERS: [u8; 4] = const_keccak_sel(b"getAllTransactionFilterers()");
pub const SEL_ARB_OWNER_SETFILTEREDFUNDSRECIPIENT: [u8; 4] = const_keccak_sel(b"setFilteredFundsRecipient(address)");
pub const SEL_ARB_OWNER_GETFILTEREDFUNDSRECIPIENT: [u8; 4] = const_keccak_sel(b"getFilteredFundsRecipient()");
pub const SEL_ARB_OWNER_SETL1BASEFEEESTIMATEINERTIA: [u8; 4] = const_keccak_sel(b"setL1BaseFeeEstimateInertia(uint64)");
pub const SEL_ARB_OWNER_SETL2BASEFEE: [u8; 4] = const_keccak_sel(b"setL2BaseFee(uint256)");
pub const SEL_ARB_OWNER_SETMINIMUML2BASEFEE: [u8; 4] = const_keccak_sel(b"setMinimumL2BaseFee(uint256)");
pub const SEL_ARB_OWNER_SETSPEEDLIMIT: [u8; 4] = const_keccak_sel(b"setSpeedLimit(uint64)");
pub const SEL_ARB_OWNER_SETMAXTXGASLIMIT: [u8; 4] = const_keccak_sel(b"setMaxTxGasLimit(uint64)");
pub const SEL_ARB_OWNER_SETMAXBLOCKGASLIMIT: [u8; 4] = const_keccak_sel(b"setMaxBlockGasLimit(uint64)");
pub const SEL_ARB_OWNER_SETL2GASPRICINGINERTIA: [u8; 4] = const_keccak_sel(b"setL2GasPricingInertia(uint64)");
pub const SEL_ARB_OWNER_SETL2GASBACKLOGTOLERANCE: [u8; 4] = const_keccak_sel(b"setL2GasBacklogTolerance(uint64)");
pub const SEL_ARB_OWNER_GETNETWORKFEEACCOUNT: [u8; 4] = const_keccak_sel(b"getNetworkFeeAccount()");
pub const SEL_ARB_OWNER_GETINFRAFEEACCOUNT: [u8; 4] = const_keccak_sel(b"getInfraFeeAccount()");
pub const SEL_ARB_OWNER_SETNETWORKFEEACCOUNT: [u8; 4] = const_keccak_sel(b"setNetworkFeeAccount(address)");
pub const SEL_ARB_OWNER_SETINFRAFEEACCOUNT: [u8; 4] = const_keccak_sel(b"setInfraFeeAccount(address)");
pub const SEL_ARB_OWNER_SCHEDULEARBOSUPGRADE: [u8; 4] = const_keccak_sel(b"scheduleArbOSUpgrade(uint64,uint64)");
pub const SEL_ARB_OWNER_SETL1PRICINGEQUILIBRATIONUNITS: [u8; 4] = const_keccak_sel(b"setL1PricingEquilibrationUnits(uint256)");
pub const SEL_ARB_OWNER_SETL1PRICINGINERTIA: [u8; 4] = const_keccak_sel(b"setL1PricingInertia(uint64)");
pub const SEL_ARB_OWNER_SETL1PRICINGREWARDRECIPIENT: [u8; 4] = const_keccak_sel(b"setL1PricingRewardRecipient(address)");
pub const SEL_ARB_OWNER_SETL1PRICINGREWARDRATE: [u8; 4] = const_keccak_sel(b"setL1PricingRewardRate(uint64)");
pub const SEL_ARB_OWNER_SETL1PRICEPERUNIT: [u8; 4] = const_keccak_sel(b"setL1PricePerUnit(uint256)");
pub const SEL_ARB_OWNER_SETPARENTGASFLOORPERTOKEN: [u8; 4] = const_keccak_sel(b"setParentGasFloorPerToken(uint64)");
pub const SEL_ARB_OWNER_SETPERBATCHGASCHARGE: [u8; 4] = const_keccak_sel(b"setPerBatchGasCharge(uint64)");
pub const SEL_ARB_OWNER_SETBROTLICOMPRESSIONLEVEL: [u8; 4] = const_keccak_sel(b"setBrotliCompressionLevel(uint64)");
pub const SEL_ARB_OWNER_SETAMORTIZEDCOSTCAPBIPS: [u8; 4] = const_keccak_sel(b"setAmortizedCostCapBips(uint64)");
pub const SEL_ARB_OWNER_RELEASEL1PRICERSURPLUSFUNDS: [u8; 4] = const_keccak_sel(b"releaseL1PricerSurplusFunds(uint64)");
pub const SEL_ARB_OWNER_SETINKPRICE: [u8; 4] = const_keccak_sel(b"setInkPrice(uint32)");
pub const SEL_ARB_OWNER_SETWASMMAXSTACKDEPTH: [u8; 4] = const_keccak_sel(b"setWasmMaxStackDepth(uint32)");
pub const SEL_ARB_OWNER_SETWASMFREEPAGES: [u8; 4] = const_keccak_sel(b"setWasmFreePages(uint16)");
pub const SEL_ARB_OWNER_SETWASMPAGEGAS: [u8; 4] = const_keccak_sel(b"setWasmPageGas(uint16)");
pub const SEL_ARB_OWNER_SETWASMPAGELIMIT: [u8; 4] = const_keccak_sel(b"setWasmPageLimit(uint32)");
pub const SEL_ARB_OWNER_SETWASMMAXSIZE: [u8; 4] = const_keccak_sel(b"setWasmMaxSize(uint32)");
pub const SEL_ARB_OWNER_SETWASMMININITGAS: [u8; 4] = const_keccak_sel(b"setWasmMinInitGas(uint8,uint16)");
pub const SEL_ARB_OWNER_SETWASMINITCOSTSCALAR: [u8; 4] = const_keccak_sel(b"setWasmInitCostScalar(uint64)");
pub const SEL_ARB_OWNER_SETWASMEXPIRYDAYS: [u8; 4] = const_keccak_sel(b"setWasmExpiryDays(uint16)");
pub const SEL_ARB_OWNER_SETWASMKEEPALIVEDAYS: [u8; 4] = const_keccak_sel(b"setWasmKeepaliveDays(uint16)");
pub const SEL_ARB_OWNER_SETWASMBLOCKCACHESIZE: [u8; 4] = const_keccak_sel(b"setWasmBlockCacheSize(uint16)");
pub const SEL_ARB_OWNER_ADDWASMCACHEMANAGER: [u8; 4] = const_keccak_sel(b"addWasmCacheManager(address)");
pub const SEL_ARB_OWNER_REMOVEWASMCACHEMANAGER: [u8; 4] = const_keccak_sel(b"removeWasmCacheManager(address)");
pub const SEL_ARB_OWNER_SETCHAINCONFIG: [u8; 4] = const_keccak_sel(b"setChainConfig(string)");
pub const SEL_ARB_OWNER_SETCALLDATAPRICEINCREASE: [u8; 4] = const_keccak_sel(b"setCalldataPriceIncrease(bool)");
pub const SEL_ARB_OWNER_SETGASBACKLOG: [u8; 4] = const_keccak_sel(b"setGasBacklog(uint64)");
pub const SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS: [u8; 4] = const_keccak_sel(b"setGasPricingConstraints(uint64[3][])");
pub const SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS: [u8; 4] = const_keccak_sel(b"setMultiGasPricingConstraints(((uint8,uint64)[],uint32,uint64,uint64)[])");
pub const SEL_ARB_OWNER_SETCOLLECTTIPS: [u8; 4] = const_keccak_sel(b"setCollectTips(bool)");
pub const SEL_ARB_OWNER_SETMAXSTYLUSCONTRACTFRAGMENTS: [u8; 4] = const_keccak_sel(b"setMaxStylusContractFragments(uint8)");
pub const SEL_ARB_OWNER_SETWASMACTIVATIONGAS: [u8; 4] = const_keccak_sel(b"setWasmActivationGas(uint64)");

pub const SEL_ARB_WASM_ACTIVATEPROGRAM: [u8; 4] = const_keccak_sel(b"activateProgram(address)");
pub const SEL_ARB_WASM_STYLUSVERSION: [u8; 4] = const_keccak_sel(b"stylusVersion()");
pub const SEL_ARB_WASM_CODEHASHVERSION: [u8; 4] = const_keccak_sel(b"codehashVersion(bytes32)");
pub const SEL_ARB_WASM_CODEHASHKEEPALIVE: [u8; 4] = const_keccak_sel(b"codehashKeepalive(bytes32)");
pub const SEL_ARB_WASM_CODEHASHASMSIZE: [u8; 4] = const_keccak_sel(b"codehashAsmSize(bytes32)");
pub const SEL_ARB_WASM_PROGRAMVERSION: [u8; 4] = const_keccak_sel(b"programVersion(address)");
pub const SEL_ARB_WASM_PROGRAMINITGAS: [u8; 4] = const_keccak_sel(b"programInitGas(address)");
pub const SEL_ARB_WASM_PROGRAMMEMORYFOOTPRINT: [u8; 4] = const_keccak_sel(b"programMemoryFootprint(address)");
pub const SEL_ARB_WASM_PROGRAMTIMELEFT: [u8; 4] = const_keccak_sel(b"programTimeLeft(address)");
pub const SEL_ARB_WASM_INKPRICE: [u8; 4] = const_keccak_sel(b"inkPrice()");
pub const SEL_ARB_WASM_MAXSTACKDEPTH: [u8; 4] = const_keccak_sel(b"maxStackDepth()");
pub const SEL_ARB_WASM_FREEPAGES: [u8; 4] = const_keccak_sel(b"freePages()");
pub const SEL_ARB_WASM_PAGEGAS: [u8; 4] = const_keccak_sel(b"pageGas()");
pub const SEL_ARB_WASM_PAGERAMP: [u8; 4] = const_keccak_sel(b"pageRamp()");
pub const SEL_ARB_WASM_PAGELIMIT: [u8; 4] = const_keccak_sel(b"pageLimit()");
pub const SEL_ARB_WASM_MININITGAS: [u8; 4] = const_keccak_sel(b"minInitGas()");
pub const SEL_ARB_WASM_INITCOSTSCALAR: [u8; 4] = const_keccak_sel(b"initCostScalar()");
pub const SEL_ARB_WASM_EXPIRYDAYS: [u8; 4] = const_keccak_sel(b"expiryDays()");
pub const SEL_ARB_WASM_KEEPALIVEDAYS: [u8; 4] = const_keccak_sel(b"keepaliveDays()");
pub const SEL_ARB_WASM_BLOCKCACHESIZE: [u8; 4] = const_keccak_sel(b"blockCacheSize()");
pub const SEL_ARB_WASM_ACTIVATIONGAS: [u8; 4] = const_keccak_sel(b"activationGas()");

pub const SEL_ARB_WASM_CACHE_ISCACHEMANAGER: [u8; 4] = const_keccak_sel(b"isCacheManager(address)");
pub const SEL_ARB_WASM_CACHE_ALLCACHEMANAGERS: [u8; 4] = const_keccak_sel(b"allCacheManagers()");
pub const SEL_ARB_WASM_CACHE_CACHECODEHASH: [u8; 4] = const_keccak_sel(b"cacheCodehash(bytes32)");
pub const SEL_ARB_WASM_CACHE_CACHEPROGRAM: [u8; 4] = const_keccak_sel(b"cacheProgram(address)");
pub const SEL_ARB_WASM_CACHE_EVICTCODEHASH: [u8; 4] = const_keccak_sel(b"evictCodehash(bytes32)");
pub const SEL_ARB_WASM_CACHE_CODEHASHISCACHED: [u8; 4] = const_keccak_sel(b"codehashIsCached(bytes32)");

pub const SEL_ARB_NATIVE_TOKEN_MANAGER_MINTNATIVETOKEN: [u8; 4] = const_keccak_sel(b"mintNativeToken(uint256)");
pub const SEL_ARB_NATIVE_TOKEN_MANAGER_BURNNATIVETOKEN: [u8; 4] = const_keccak_sel(b"burnNativeToken(uint256)");

pub const SEL_ARB_DEBUG_BECOMECHAINOWNER: [u8; 4] = const_keccak_sel(b"becomeChainOwner()");
pub const SEL_ARB_DEBUG_OVERWRITECONTRACTCODE: [u8; 4] = const_keccak_sel(b"overwriteContractCode(address,bytes)");
pub const SEL_ARB_DEBUG_EVENTS: [u8; 4] = const_keccak_sel(b"events(bool,bytes32)");
pub const SEL_ARB_DEBUG_EVENTSVIEW: [u8; 4] = const_keccak_sel(b"eventsView()");
pub const SEL_ARB_DEBUG_CUSTOMREVERT: [u8; 4] = const_keccak_sel(b"customRevert(uint64)");
pub const SEL_ARB_DEBUG_PANIC: [u8; 4] = const_keccak_sel(b"panic()");
pub const SEL_ARB_DEBUG_LEGACYERROR: [u8; 4] = const_keccak_sel(b"legacyError()");

fn make_fn_arb_sys_arb_block_number() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_ARBBLOCKNUMBER
    )
}

fn make_fn_arb_sys_arb_block_hash(arbBlockNum: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_SYS_ARBBLOCKHASH,
        arbBlockNum.0
    )
}

fn make_fn_arb_sys_arb_chain_i_d() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_ARBCHAINID
    )
}

fn make_fn_arb_sys_arb_o_s_version() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_ARBOSVERSION
    )
}

fn make_fn_arb_sys_get_storage_gas_available() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_GETSTORAGEGASAVAILABLE
    )
}

fn make_fn_arb_sys_is_top_level_call() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_ISTOPLEVELCALL
    )
}

fn make_fn_arb_sys_map_l1_sender_contract_address_to_l2_alias(sender: [u8; 20], unused: [u8; 20]) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_SYS_MAPL1SENDERCONTRACTADDRESSTOL2ALIAS,
        leftpad_addr(sender),
        leftpad_addr(unused)
    )
}

fn make_fn_arb_sys_was_my_callers_address_aliased() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_WASMYCALLERSADDRESSALIASED
    )
}

fn make_fn_arb_sys_my_callers_address_without_aliasing() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_MYCALLERSADDRESSWITHOUTALIASING
    )
}

fn make_fn_arb_sys_withdraw_eth(destination: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_SYS_WITHDRAWETH,
        leftpad_addr(destination)
    )
}

fn make_fn_arb_sys_send_tx_to_l1<const N: usize, const PAD: usize, const TOTAL: usize>(
    destination: [u8; 20],
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 100 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_SYS_SENDTXTOL1,
        leftpad_addr(destination),
        leftpad_usize(64),
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_sys_send_merkle_tree_state() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_SYS_SENDMERKLETREESTATE
    )
}

fn make_fn_arb_info_get_balance(account: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_INFO_GETBALANCE,
        leftpad_addr(account)
    )
}

fn make_fn_arb_info_get_code(account: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_INFO_GETCODE,
        leftpad_addr(account)
    )
}

fn make_fn_arb_address_table_address_exists(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_ADDRESSEXISTS,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_address_table_compress(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_COMPRESS,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_address_table_decompress<const N: usize, const PAD: usize, const TOTAL: usize>(
    offset: U,
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 100 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_DECOMPRESS,
        leftpad_usize(64),
        offset.0,
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_address_table_lookup(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_LOOKUP,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_address_table_lookup_index(index: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_LOOKUPINDEX,
        index.0
    )
}

fn make_fn_arb_address_table_register(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_REGISTER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_address_table_size() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_ADDRESS_TABLE_SIZE
    )
}

fn make_fn_arb_function_table_upload<const N: usize, const PAD: usize, const TOTAL: usize>(
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 68 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_FUNCTION_TABLE_UPLOAD,
        leftpad_usize(32),
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_function_table_size(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_FUNCTION_TABLE_SIZE,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_function_table_get(addr: [u8; 20], index: U) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_FUNCTION_TABLE_GET,
        leftpad_addr(addr),
        index.0
    )
}

fn make_fn_arbos_test_burn_arb_gas(gasAmount: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARBOS_TEST_BURNARBGAS,
        gasAmount.0
    )
}

fn make_fn_arb_owner_public_is_chain_owner(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_ISCHAINOWNER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_owner_public_rectify_chain_owner(ownerToRectify: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_RECTIFYCHAINOWNER,
        leftpad_addr(ownerToRectify)
    )
}

fn make_fn_arb_owner_public_get_all_chain_owners() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETALLCHAINOWNERS
    )
}

fn make_fn_arb_owner_public_get_native_token_management_from() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETNATIVETOKENMANAGEMENTFROM
    )
}

fn make_fn_arb_owner_public_is_native_token_owner(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_ISNATIVETOKENOWNER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_owner_public_get_all_native_token_owners() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETALLNATIVETOKENOWNERS
    )
}

fn make_fn_arb_owner_public_get_transaction_filtering_from() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETTRANSACTIONFILTERINGFROM
    )
}

fn make_fn_arb_owner_public_is_transaction_filterer(filterer: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_ISTRANSACTIONFILTERER,
        leftpad_addr(filterer)
    )
}

fn make_fn_arb_owner_public_get_all_transaction_filterers() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETALLTRANSACTIONFILTERERS
    )
}

fn make_fn_arb_owner_public_get_filtered_funds_recipient() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETFILTEREDFUNDSRECIPIENT
    )
}

fn make_fn_arb_owner_public_get_network_fee_account() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETNETWORKFEEACCOUNT
    )
}

fn make_fn_arb_owner_public_get_infra_fee_account() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETINFRAFEEACCOUNT
    )
}

fn make_fn_arb_owner_public_get_brotli_compression_level() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETBROTLICOMPRESSIONLEVEL
    )
}

fn make_fn_arb_owner_public_get_parent_gas_floor_per_token() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETPARENTGASFLOORPERTOKEN
    )
}

fn make_fn_arb_owner_public_get_scheduled_upgrade() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETSCHEDULEDUPGRADE
    )
}

fn make_fn_arb_owner_public_is_calldata_price_increase_enabled() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_ISCALLDATAPRICEINCREASEENABLED
    )
}

fn make_fn_arb_owner_public_get_collect_tips() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETCOLLECTTIPS
    )
}

fn make_fn_arb_owner_public_get_max_stylus_contract_fragments() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_PUBLIC_GETMAXSTYLUSCONTRACTFRAGMENTS
    )
}

fn make_fn_arb_gas_info_get_prices_in_wei_with_aggregator(aggregator: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPRICESINWEIWITHAGGREGATOR,
        leftpad_addr(aggregator)
    )
}

fn make_fn_arb_gas_info_get_prices_in_wei() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPRICESINWEI
    )
}

fn make_fn_arb_gas_info_get_prices_in_arb_gas_with_aggregator(aggregator: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPRICESINARBGASWITHAGGREGATOR,
        leftpad_addr(aggregator)
    )
}

fn make_fn_arb_gas_info_get_prices_in_arb_gas() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPRICESINARBGAS
    )
}

fn make_fn_arb_gas_info_get_gas_accounting_params() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETGASACCOUNTINGPARAMS
    )
}

fn make_fn_arb_gas_info_get_max_tx_gas_limit() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETMAXTXGASLIMIT
    )
}

fn make_fn_arb_gas_info_get_minimum_gas_price() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETMINIMUMGASPRICE
    )
}

fn make_fn_arb_gas_info_get_l1_base_fee_estimate() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1BASEFEEESTIMATE
    )
}

fn make_fn_arb_gas_info_get_l1_base_fee_estimate_inertia() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1BASEFEEESTIMATEINERTIA
    )
}

fn make_fn_arb_gas_info_get_l1_reward_rate() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1REWARDRATE
    )
}

fn make_fn_arb_gas_info_get_l1_reward_recipient() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1REWARDRECIPIENT
    )
}

fn make_fn_arb_gas_info_get_l1_gas_price_estimate() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1GASPRICEESTIMATE
    )
}

fn make_fn_arb_gas_info_get_current_tx_l1_gas_fees() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETCURRENTTXL1GASFEES
    )
}

fn make_fn_arb_gas_info_get_gas_backlog() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETGASBACKLOG
    )
}

fn make_fn_arb_gas_info_get_pricing_inertia() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPRICINGINERTIA
    )
}

fn make_fn_arb_gas_info_get_gas_backlog_tolerance() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETGASBACKLOGTOLERANCE
    )
}

fn make_fn_arb_gas_info_get_l1_pricing_surplus() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1PRICINGSURPLUS
    )
}

fn make_fn_arb_gas_info_get_per_batch_gas_charge() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETPERBATCHGASCHARGE
    )
}

fn make_fn_arb_gas_info_get_amortized_cost_cap_bips() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETAMORTIZEDCOSTCAPBIPS
    )
}

fn make_fn_arb_gas_info_get_l1_fees_available() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1FEESAVAILABLE
    )
}

fn make_fn_arb_gas_info_get_l1_pricing_equilibration_units() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1PRICINGEQUILIBRATIONUNITS
    )
}

fn make_fn_arb_gas_info_get_last_l1_pricing_update_time() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETLASTL1PRICINGUPDATETIME
    )
}

fn make_fn_arb_gas_info_get_l1_pricing_funds_due_for_rewards() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1PRICINGFUNDSDUEFORREWARDS
    )
}

fn make_fn_arb_gas_info_get_l1_pricing_units_since_update() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETL1PRICINGUNITSSINCEUPDATE
    )
}

fn make_fn_arb_gas_info_get_last_l1_pricing_surplus() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETLASTL1PRICINGSURPLUS
    )
}

fn make_fn_arb_gas_info_get_max_block_gas_limit() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETMAXBLOCKGASLIMIT
    )
}

fn make_fn_arb_gas_info_get_gas_pricing_constraints() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETGASPRICINGCONSTRAINTS
    )
}

fn make_fn_arb_gas_info_get_multi_gas_pricing_constraints() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETMULTIGASPRICINGCONSTRAINTS
    )
}

fn make_fn_arb_gas_info_get_multi_gas_base_fee() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_GAS_INFO_GETMULTIGASBASEFEE
    )
}

fn make_fn_arb_aggregator_get_preferred_aggregator(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_GETPREFERREDAGGREGATOR,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_aggregator_get_default_aggregator() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_GETDEFAULTAGGREGATOR
    )
}

fn make_fn_arb_aggregator_get_batch_posters() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_GETBATCHPOSTERS
    )
}

fn make_fn_arb_aggregator_add_batch_poster(newBatchPoster: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_ADDBATCHPOSTER,
        leftpad_addr(newBatchPoster)
    )
}

fn make_fn_arb_aggregator_get_fee_collector(batchPoster: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_GETFEECOLLECTOR,
        leftpad_addr(batchPoster)
    )
}

fn make_fn_arb_aggregator_set_fee_collector(batchPoster: [u8; 20], newFeeCollector: [u8; 20]) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_SETFEECOLLECTOR,
        leftpad_addr(batchPoster),
        leftpad_addr(newFeeCollector)
    )
}

fn make_fn_arb_aggregator_get_tx_base_fee(aggregator: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_GETTXBASEFEE,
        leftpad_addr(aggregator)
    )
}

fn make_fn_arb_aggregator_set_tx_base_fee(aggregator: [u8; 20], feeInL1Gas: U) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_AGGREGATOR_SETTXBASEFEE,
        leftpad_addr(aggregator),
        feeInL1Gas.0
    )
}

fn make_fn_arb_retryable_tx_redeem(ticketId: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_REDEEM,
        ticketId
    )
}

fn make_fn_arb_retryable_tx_get_lifetime() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_GETLIFETIME
    )
}

fn make_fn_arb_retryable_tx_get_timeout(ticketId: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_GETTIMEOUT,
        ticketId
    )
}

fn make_fn_arb_retryable_tx_keepalive(ticketId: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_KEEPALIVE,
        ticketId
    )
}

fn make_fn_arb_retryable_tx_get_beneficiary(ticketId: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_GETBENEFICIARY,
        ticketId
    )
}

fn make_fn_arb_retryable_tx_cancel(ticketId: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_CANCEL,
        ticketId
    )
}

fn make_fn_arb_retryable_tx_get_current_redeemer() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_GETCURRENTREDEEMER
    )
}

fn make_fn_arb_retryable_tx_submit_retryable<const N: usize, const PAD: usize, const TOTAL: usize>(
    requestId: [u8; 32],
    l1BaseFee: U,
    deposit: U,
    callvalue: U,
    gasFeeCap: U,
    gasLimit: u64,
    maxSubmissionFee: U,
    feeRefundAddress: [u8; 20],
    beneficiary: [u8; 20],
    retryTo: [u8; 20],
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 388 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_RETRYABLE_TX_SUBMITRETRYABLE,
        requestId,
        l1BaseFee.0,
        deposit.0,
        callvalue.0,
        gasFeeCap.0,
        leftpad_u64(gasLimit),
        maxSubmissionFee.0,
        leftpad_addr(feeRefundAddress),
        leftpad_addr(beneficiary),
        leftpad_addr(retryTo),
        leftpad_usize(352),
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_statistics_get_stats() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_STATISTICS_GETSTATS
    )
}

fn make_fn_arb_owner_add_chain_owner(newOwner: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ADDCHAINOWNER,
        leftpad_addr(newOwner)
    )
}

fn make_fn_arb_owner_remove_chain_owner(owner: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_REMOVECHAINOWNER,
        leftpad_addr(owner)
    )
}

fn make_fn_arb_owner_is_chain_owner(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ISCHAINOWNER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_owner_get_all_chain_owners() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETALLCHAINOWNERS
    )
}

fn make_fn_arb_owner_set_native_token_management_from(newTimestamp: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETNATIVETOKENMANAGEMENTFROM,
        leftpad_u64(newTimestamp)
    )
}

fn make_fn_arb_owner_set_transaction_filtering_from(newTimestamp: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETTRANSACTIONFILTERINGFROM,
        leftpad_u64(newTimestamp)
    )
}

fn make_fn_arb_owner_add_native_token_owner(newOwner: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ADDNATIVETOKENOWNER,
        leftpad_addr(newOwner)
    )
}

fn make_fn_arb_owner_remove_native_token_owner(owner: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_REMOVENATIVETOKENOWNER,
        leftpad_addr(owner)
    )
}

fn make_fn_arb_owner_is_native_token_owner(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ISNATIVETOKENOWNER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_owner_get_all_native_token_owners() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETALLNATIVETOKENOWNERS
    )
}

fn make_fn_arb_owner_add_transaction_filterer(newFilterer: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ADDTRANSACTIONFILTERER,
        leftpad_addr(newFilterer)
    )
}

fn make_fn_arb_owner_remove_transaction_filterer(filterer: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_REMOVETRANSACTIONFILTERER,
        leftpad_addr(filterer)
    )
}

fn make_fn_arb_owner_is_transaction_filterer(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ISTRANSACTIONFILTERER,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_owner_get_all_transaction_filterers() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETALLTRANSACTIONFILTERERS
    )
}

fn make_fn_arb_owner_set_filtered_funds_recipient(receiver: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETFILTEREDFUNDSRECIPIENT,
        leftpad_addr(receiver)
    )
}

fn make_fn_arb_owner_get_filtered_funds_recipient() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETFILTEREDFUNDSRECIPIENT
    )
}

fn make_fn_arb_owner_set_l1_base_fee_estimate_inertia(inertia: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1BASEFEEESTIMATEINERTIA,
        leftpad_u64(inertia)
    )
}

fn make_fn_arb_owner_set_l2_base_fee(priceInWei: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL2BASEFEE,
        priceInWei.0
    )
}

fn make_fn_arb_owner_set_minimum_l2_base_fee(priceInWei: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETMINIMUML2BASEFEE,
        priceInWei.0
    )
}

fn make_fn_arb_owner_set_speed_limit(speedLimit: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETSPEEDLIMIT,
        leftpad_u64(speedLimit)
    )
}

fn make_fn_arb_owner_set_max_tx_gas_limit(maxTxGasLimit: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETMAXTXGASLIMIT,
        leftpad_u64(maxTxGasLimit)
    )
}

fn make_fn_arb_owner_set_max_block_gas_limit(maxBlockGasLimit: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETMAXBLOCKGASLIMIT,
        leftpad_u64(maxBlockGasLimit)
    )
}

fn make_fn_arb_owner_set_l2_gas_pricing_inertia(inertia: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL2GASPRICINGINERTIA,
        leftpad_u64(inertia)
    )
}

fn make_fn_arb_owner_set_l2_gas_backlog_tolerance(tolerance: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL2GASBACKLOGTOLERANCE,
        leftpad_u64(tolerance)
    )
}

fn make_fn_arb_owner_get_network_fee_account() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETNETWORKFEEACCOUNT
    )
}

fn make_fn_arb_owner_get_infra_fee_account() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_OWNER_GETINFRAFEEACCOUNT
    )
}

fn make_fn_arb_owner_set_network_fee_account(newNetworkFeeAccount: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETNETWORKFEEACCOUNT,
        leftpad_addr(newNetworkFeeAccount)
    )
}

fn make_fn_arb_owner_set_infra_fee_account(newInfraFeeAccount: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETINFRAFEEACCOUNT,
        leftpad_addr(newInfraFeeAccount)
    )
}

fn make_fn_arb_owner_schedule_arb_o_s_upgrade(newVersion: u64, timestamp: u64) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_OWNER_SCHEDULEARBOSUPGRADE,
        leftpad_u64(newVersion),
        leftpad_u64(timestamp)
    )
}

fn make_fn_arb_owner_set_l1_pricing_equilibration_units(units: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1PRICINGEQUILIBRATIONUNITS,
        units.0
    )
}

fn make_fn_arb_owner_set_l1_pricing_inertia(inertia: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1PRICINGINERTIA,
        leftpad_u64(inertia)
    )
}

fn make_fn_arb_owner_set_l1_pricing_reward_recipient(recipient: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1PRICINGREWARDRECIPIENT,
        leftpad_addr(recipient)
    )
}

fn make_fn_arb_owner_set_l1_pricing_reward_rate(recipientRate: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1PRICINGREWARDRATE,
        leftpad_u64(recipientRate)
    )
}

fn make_fn_arb_owner_set_l1_price_per_unit(pricePerUnit: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETL1PRICEPERUNIT,
        pricePerUnit.0
    )
}

fn make_fn_arb_owner_set_parent_gas_floor_per_token(floorPerToken: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETPARENTGASFLOORPERTOKEN,
        leftpad_u64(floorPerToken)
    )
}

fn make_fn_arb_owner_set_per_batch_gas_charge(charge: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETPERBATCHGASCHARGE,
        leftpad_u64(charge)
    )
}

fn make_fn_arb_owner_set_brotli_compression_level(level: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETBROTLICOMPRESSIONLEVEL,
        leftpad_u64(level)
    )
}

fn make_fn_arb_owner_set_amortized_cost_cap_bips(capBips: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETAMORTIZEDCOSTCAPBIPS,
        leftpad_u64(capBips)
    )
}

fn make_fn_arb_owner_release_l1_pricer_surplus_funds(surplusToRelease: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_RELEASEL1PRICERSURPLUSFUNDS,
        leftpad_u64(surplusToRelease)
    )
}

fn make_fn_arb_owner_set_ink_price(price: u32) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETINKPRICE,
        leftpad_u32(price)
    )
}

fn make_fn_arb_owner_set_wasm_max_stack_depth(depth: u32) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMMAXSTACKDEPTH,
        leftpad_u32(depth)
    )
}

fn make_fn_arb_owner_set_wasm_free_pages(amount: u16) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMFREEPAGES,
        leftpad_u16(amount)
    )
}

fn make_fn_arb_owner_set_wasm_page_gas(amount: u16) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMPAGEGAS,
        leftpad_u16(amount)
    )
}

fn make_fn_arb_owner_set_wasm_page_limit(limit: u32) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMPAGELIMIT,
        leftpad_u32(limit)
    )
}

fn make_fn_arb_owner_set_wasm_max_size(size: u32) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMMAXSIZE,
        leftpad_u32(size)
    )
}

fn make_fn_arb_owner_set_wasm_min_init_gas(gas: u8, cached: u16) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMMININITGAS,
        leftpad_u8(gas),
        leftpad_u16(cached)
    )
}

fn make_fn_arb_owner_set_wasm_init_cost_scalar(percent: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMINITCOSTSCALAR,
        leftpad_u64(percent)
    )
}

fn make_fn_arb_owner_set_wasm_expiry_days(days: u16) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMEXPIRYDAYS,
        leftpad_u16(days)
    )
}

fn make_fn_arb_owner_set_wasm_keepalive_days(days: u16) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMKEEPALIVEDAYS,
        leftpad_u16(days)
    )
}

fn make_fn_arb_owner_set_wasm_block_cache_size(size: u16) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMBLOCKCACHESIZE,
        leftpad_u16(size)
    )
}

fn make_fn_arb_owner_add_wasm_cache_manager(manager: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_ADDWASMCACHEMANAGER,
        leftpad_addr(manager)
    )
}

fn make_fn_arb_owner_remove_wasm_cache_manager(manager: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_REMOVEWASMCACHEMANAGER,
        leftpad_addr(manager)
    )
}

fn make_fn_arb_owner_set_chain_config<const N: usize, const PAD: usize, const TOTAL: usize>(
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 68 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_OWNER_SETCHAINCONFIG,
        leftpad_usize(32),
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_owner_set_calldata_price_increase(enable: bool) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETCALLDATAPRICEINCREASE,
        leftpad_bool(enable)
    )
}

fn make_fn_arb_owner_set_gas_backlog(backlog: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETGASBACKLOG,
        leftpad_u64(backlog)
    )
}

fn make_fn_arb_owner_set_gas_pricing_constraints<const N: usize, const TOTAL: usize>(tail: &[u8; N]) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 4 + N, "total cap not consistent");
    concat_arrays!(SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS, *tail)
}

fn make_fn_arb_owner_set_multi_gas_pricing_constraints<const N: usize, const TOTAL: usize>(tail: &[u8; N]) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 4 + N, "total cap not consistent");
    concat_arrays!(SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS, *tail)
}

fn make_fn_arb_owner_set_collect_tips(collectTips: bool) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETCOLLECTTIPS,
        leftpad_bool(collectTips)
    )
}

fn make_fn_arb_owner_set_max_stylus_contract_fragments(maxFragments: u8) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETMAXSTYLUSCONTRACTFRAGMENTS,
        leftpad_u8(maxFragments)
    )
}

fn make_fn_arb_owner_set_wasm_activation_gas(gas: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_OWNER_SETWASMACTIVATIONGAS,
        leftpad_u64(gas)
    )
}

fn make_fn_arb_wasm_activate_program(program: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_ACTIVATEPROGRAM,
        leftpad_addr(program)
    )
}

fn make_fn_arb_wasm_stylus_version() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_STYLUSVERSION
    )
}

fn make_fn_arb_wasm_codehash_version(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CODEHASHVERSION,
        codehash
    )
}

fn make_fn_arb_wasm_codehash_keepalive(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CODEHASHKEEPALIVE,
        codehash
    )
}

fn make_fn_arb_wasm_codehash_asm_size(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CODEHASHASMSIZE,
        codehash
    )
}

fn make_fn_arb_wasm_program_version(program: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_PROGRAMVERSION,
        leftpad_addr(program)
    )
}

fn make_fn_arb_wasm_program_init_gas(program: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_PROGRAMINITGAS,
        leftpad_addr(program)
    )
}

fn make_fn_arb_wasm_program_memory_footprint(program: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_PROGRAMMEMORYFOOTPRINT,
        leftpad_addr(program)
    )
}

fn make_fn_arb_wasm_program_time_left(program: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_PROGRAMTIMELEFT,
        leftpad_addr(program)
    )
}

fn make_fn_arb_wasm_ink_price() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_INKPRICE
    )
}

fn make_fn_arb_wasm_max_stack_depth() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_MAXSTACKDEPTH
    )
}

fn make_fn_arb_wasm_free_pages() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_FREEPAGES
    )
}

fn make_fn_arb_wasm_page_gas() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_PAGEGAS
    )
}

fn make_fn_arb_wasm_page_ramp() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_PAGERAMP
    )
}

fn make_fn_arb_wasm_page_limit() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_PAGELIMIT
    )
}

fn make_fn_arb_wasm_min_init_gas() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_MININITGAS
    )
}

fn make_fn_arb_wasm_init_cost_scalar() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_INITCOSTSCALAR
    )
}

fn make_fn_arb_wasm_expiry_days() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_EXPIRYDAYS
    )
}

fn make_fn_arb_wasm_keepalive_days() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_KEEPALIVEDAYS
    )
}

fn make_fn_arb_wasm_block_cache_size() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_BLOCKCACHESIZE
    )
}

fn make_fn_arb_wasm_activation_gas() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_ACTIVATIONGAS
    )
}

fn make_fn_arb_wasm_cache_is_cache_manager(manager: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_ISCACHEMANAGER,
        leftpad_addr(manager)
    )
}

fn make_fn_arb_wasm_cache_all_cache_managers() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_ALLCACHEMANAGERS
    )
}

fn make_fn_arb_wasm_cache_cache_codehash(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_CACHECODEHASH,
        codehash
    )
}

fn make_fn_arb_wasm_cache_cache_program(addr: [u8; 20]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_CACHEPROGRAM,
        leftpad_addr(addr)
    )
}

fn make_fn_arb_wasm_cache_evict_codehash(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_EVICTCODEHASH,
        codehash
    )
}

fn make_fn_arb_wasm_cache_codehash_is_cached(codehash: [u8; 32]) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_WASM_CACHE_CODEHASHISCACHED,
        codehash
    )
}

fn make_fn_arb_native_token_manager_mint_native_token(amount: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_NATIVE_TOKEN_MANAGER_MINTNATIVETOKEN,
        amount.0
    )
}

fn make_fn_arb_native_token_manager_burn_native_token(amount: U) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_NATIVE_TOKEN_MANAGER_BURNNATIVETOKEN,
        amount.0
    )
}

fn make_fn_arb_debug_become_chain_owner() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_DEBUG_BECOMECHAINOWNER
    )
}

fn make_fn_arb_debug_overwrite_contract_code<const N: usize, const PAD: usize, const TOTAL: usize>(
    target: [u8; 20],
    buf: &[u8; N],
) -> [u8; TOTAL] {
    assert_eq!(TOTAL, 100 + N + PAD, "total cap not consistent");
    assert_eq!(PAD, ((N + 31) / 32) * 32 - N, "pad cap not consistent");
    concat_arrays!(
        SEL_ARB_DEBUG_OVERWRITECONTRACTCODE,
        leftpad_addr(target),
        leftpad_usize(64),
        leftpad_usize(N),
        *buf,
        [0u8; PAD]
    )
}

fn make_fn_arb_debug_events(flag: bool, value: [u8; 32]) -> [u8; 68] {
    concat_arrays!(
        SEL_ARB_DEBUG_EVENTS,
        leftpad_bool(flag),
        value
    )
}

fn make_fn_arb_debug_events_view() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_DEBUG_EVENTSVIEW
    )
}

fn make_fn_arb_debug_custom_revert(number: u64) -> [u8; 36] {
    concat_arrays!(
        SEL_ARB_DEBUG_CUSTOMREVERT,
        leftpad_u64(number)
    )
}

fn make_fn_arb_debug_panic() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_DEBUG_PANIC
    )
}

fn make_fn_arb_debug_legacy_error() -> [u8; 4] {
    concat_arrays!(
        SEL_ARB_DEBUG_LEGACYERROR
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_macro::sol;
    use alloy_primitives::{Address as AAddress, Bytes, FixedBytes, U256 as AU};
    use alloy_sol_types::SolCall;
    use proptest::prelude::*;

    mod p_arb_sys {
        use super::*;
        sol! {
            // arb_sys
            function arbBlockNumber() external;
            function arbBlockHash(uint256 arbBlockNum) external;
            function arbChainID() external;
            function arbOSVersion() external;
            function getStorageGasAvailable() external;
            function isTopLevelCall() external;
            function mapL1SenderContractAddressToL2Alias(address sender, address unused) external;
            function wasMyCallersAddressAliased() external;
            function myCallersAddressWithoutAliasing() external;
            function withdrawEth(address destination) external;
            function sendTxToL1(address destination, bytes data) external;
            function sendMerkleTreeState() external;
        }
    }

    mod p_arb_info {
        use super::*;
        sol! {
            // arb_info
            function getBalance(address account) external;
            function getCode(address account) external;
        }
    }

    mod p_arb_address_table {
        use super::*;
        sol! {
            // arb_address_table
            function addressExists(address addr) external;
            function compress(address addr) external;
            function decompress(bytes buf, uint256 offset) external;
            function lookup(address addr) external;
            function lookupIndex(uint256 index) external;
            function register(address addr) external;
            function size() external;
        }
    }

    mod p_arb_function_table {
        use super::*;
        sol! {
            // arb_function_table
            function upload(bytes buf) external;
            function size(address addr) external;
            function get(address addr, uint256 index) external;
        }
    }

    mod p_arbos_test {
        use super::*;
        sol! {
            // arbos_test
            function burnArbGas(uint256 gasAmount) external;
        }
    }

    mod p_arb_owner_public {
        use super::*;
        sol! {
            // arb_owner_public
            function isChainOwner(address addr) external;
            function rectifyChainOwner(address ownerToRectify) external;
            function getAllChainOwners() external;
            function getNativeTokenManagementFrom() external;
            function isNativeTokenOwner(address addr) external;
            function getAllNativeTokenOwners() external;
            function getTransactionFilteringFrom() external;
            function isTransactionFilterer(address filterer) external;
            function getAllTransactionFilterers() external;
            function getFilteredFundsRecipient() external;
            function getNetworkFeeAccount() external;
            function getInfraFeeAccount() external;
            function getBrotliCompressionLevel() external;
            function getParentGasFloorPerToken() external;
            function getScheduledUpgrade() external;
            function isCalldataPriceIncreaseEnabled() external;
            function getCollectTips() external;
            function getMaxStylusContractFragments() external;
        }
    }

    mod p_arb_gas_info {
        use super::*;
        sol! {
            // arb_gas_info
            function getPricesInWeiWithAggregator(address aggregator) external;
            function getPricesInWei() external;
            function getPricesInArbGasWithAggregator(address aggregator) external;
            function getPricesInArbGas() external;
            function getGasAccountingParams() external;
            function getMaxTxGasLimit() external;
            function getMinimumGasPrice() external;
            function getL1BaseFeeEstimate() external;
            function getL1BaseFeeEstimateInertia() external;
            function getL1RewardRate() external;
            function getL1RewardRecipient() external;
            function getL1GasPriceEstimate() external;
            function getCurrentTxL1GasFees() external;
            function getGasBacklog() external;
            function getPricingInertia() external;
            function getGasBacklogTolerance() external;
            function getL1PricingSurplus() external;
            function getPerBatchGasCharge() external;
            function getAmortizedCostCapBips() external;
            function getL1FeesAvailable() external;
            function getL1PricingEquilibrationUnits() external;
            function getLastL1PricingUpdateTime() external;
            function getL1PricingFundsDueForRewards() external;
            function getL1PricingUnitsSinceUpdate() external;
            function getLastL1PricingSurplus() external;
            function getMaxBlockGasLimit() external;
            function getGasPricingConstraints() external;
            function getMultiGasPricingConstraints() external;
            function getMultiGasBaseFee() external;
        }
    }

    mod p_arb_aggregator {
        use super::*;
        sol! {
            // arb_aggregator
            function getPreferredAggregator(address addr) external;
            function getDefaultAggregator() external;
            function getBatchPosters() external;
            function addBatchPoster(address newBatchPoster) external;
            function getFeeCollector(address batchPoster) external;
            function setFeeCollector(address batchPoster, address newFeeCollector) external;
            function getTxBaseFee(address aggregator) external;
            function setTxBaseFee(address aggregator, uint256 feeInL1Gas) external;
        }
    }

    mod p_arb_retryable_tx {
        use super::*;
        sol! {
            // arb_retryable_tx
            function redeem(bytes32 ticketId) external;
            function getLifetime() external;
            function getTimeout(bytes32 ticketId) external;
            function keepalive(bytes32 ticketId) external;
            function getBeneficiary(bytes32 ticketId) external;
            function cancel(bytes32 ticketId) external;
            function getCurrentRedeemer() external;
            function submitRetryable(bytes32 requestId, uint256 l1BaseFee, uint256 deposit, uint256 callvalue, uint256 gasFeeCap, uint64 gasLimit, uint256 maxSubmissionFee, address feeRefundAddress, address beneficiary, address retryTo, bytes retryData) external;
        }
    }

    mod p_arb_statistics {
        use super::*;
        sol! {
            // arb_statistics
            function getStats() external;
        }
    }

    mod p_arb_owner {
        use super::*;
        sol! {
            // arb_owner
            function addChainOwner(address newOwner) external;
            function removeChainOwner(address owner) external;
            function isChainOwner(address addr) external;
            function getAllChainOwners() external;
            function setNativeTokenManagementFrom(uint64 newTimestamp) external;
            function setTransactionFilteringFrom(uint64 newTimestamp) external;
            function addNativeTokenOwner(address newOwner) external;
            function removeNativeTokenOwner(address owner) external;
            function isNativeTokenOwner(address addr) external;
            function getAllNativeTokenOwners() external;
            function addTransactionFilterer(address newFilterer) external;
            function removeTransactionFilterer(address filterer) external;
            function isTransactionFilterer(address addr) external;
            function getAllTransactionFilterers() external;
            function setFilteredFundsRecipient(address receiver) external;
            function getFilteredFundsRecipient() external;
            function setL1BaseFeeEstimateInertia(uint64 inertia) external;
            function setL2BaseFee(uint256 priceInWei) external;
            function setMinimumL2BaseFee(uint256 priceInWei) external;
            function setSpeedLimit(uint64 speedLimit) external;
            function setMaxTxGasLimit(uint64 maxTxGasLimit) external;
            function setMaxBlockGasLimit(uint64 maxBlockGasLimit) external;
            function setL2GasPricingInertia(uint64 inertia) external;
            function setL2GasBacklogTolerance(uint64 tolerance) external;
            function getNetworkFeeAccount() external;
            function getInfraFeeAccount() external;
            function setNetworkFeeAccount(address newNetworkFeeAccount) external;
            function setInfraFeeAccount(address newInfraFeeAccount) external;
            function scheduleArbOSUpgrade(uint64 newVersion, uint64 timestamp) external;
            function setL1PricingEquilibrationUnits(uint256 units) external;
            function setL1PricingInertia(uint64 inertia) external;
            function setL1PricingRewardRecipient(address recipient) external;
            function setL1PricingRewardRate(uint64 recipientRate) external;
            function setL1PricePerUnit(uint256 pricePerUnit) external;
            function setParentGasFloorPerToken(uint64 floorPerToken) external;
            function setPerBatchGasCharge(uint64 charge) external;
            function setBrotliCompressionLevel(uint64 level) external;
            function setAmortizedCostCapBips(uint64 capBips) external;
            function releaseL1PricerSurplusFunds(uint64 surplusToRelease) external;
            function setInkPrice(uint32 price) external;
            function setWasmMaxStackDepth(uint32 depth) external;
            function setWasmFreePages(uint16 amount) external;
            function setWasmPageGas(uint16 amount) external;
            function setWasmPageLimit(uint32 limit) external;
            function setWasmMaxSize(uint32 size) external;
            function setWasmMinInitGas(uint8 gas, uint16 cached) external;
            function setWasmInitCostScalar(uint64 percent) external;
            function setWasmExpiryDays(uint16 days) external;
            function setWasmKeepaliveDays(uint16 days) external;
            function setWasmBlockCacheSize(uint16 size) external;
            function addWasmCacheManager(address manager) external;
            function removeWasmCacheManager(address manager) external;
            function setChainConfig(string chainConfig) external;
            function setCalldataPriceIncrease(bool enable) external;
            function setGasBacklog(uint64 backlog) external;
            function setGasPricingConstraints(uint64[3][] constraints) external;
            function setMultiGasPricingConstraints(ResourceConstraint[] constraints) external;
            function setCollectTips(bool collectTips) external;
            function setMaxStylusContractFragments(uint8 maxFragments) external;
            function setWasmActivationGas(uint64 gas) external;
            // Types backing the multi-gas pricing setters.
            struct WeightedResource {
                uint8 resource;
                uint64 weight;
            }
            struct ResourceConstraint {
                WeightedResource[] resources;
                uint32 adjustmentWindowSecs;
                uint64 targetPerSec;
                uint64 backlog;
            }
        }
    }

    mod p_arb_wasm {
        use super::*;
        sol! {
            // arb_wasm
            function activateProgram(address program) external;
            function stylusVersion() external;
            function codehashVersion(bytes32 codehash) external;
            function codehashKeepalive(bytes32 codehash) external;
            function codehashAsmSize(bytes32 codehash) external;
            function programVersion(address program) external;
            function programInitGas(address program) external;
            function programMemoryFootprint(address program) external;
            function programTimeLeft(address program) external;
            function inkPrice() external;
            function maxStackDepth() external;
            function freePages() external;
            function pageGas() external;
            function pageRamp() external;
            function pageLimit() external;
            function minInitGas() external;
            function initCostScalar() external;
            function expiryDays() external;
            function keepaliveDays() external;
            function blockCacheSize() external;
            function activationGas() external;
        }
    }

    mod p_arb_wasm_cache {
        use super::*;
        sol! {
            // arb_wasm_cache
            function isCacheManager(address manager) external;
            function allCacheManagers() external;
            function cacheCodehash(bytes32 codehash) external;
            function cacheProgram(address addr) external;
            function evictCodehash(bytes32 codehash) external;
            function codehashIsCached(bytes32 codehash) external;
        }
    }

    mod p_arb_native_token_manager {
        use super::*;
        sol! {
            // arb_native_token_manager
            function mintNativeToken(uint256 amount) external;
            function burnNativeToken(uint256 amount) external;
        }
    }

    mod p_arb_debug {
        use super::*;
        sol! {
            // arb_debug
            function becomeChainOwner() external;
            function overwriteContractCode(address target, bytes newCode) external;
            function events(bool flag, bytes32 value) external;
            function eventsView() external;
            function customRevert(uint64 number) external;
            function panic() external;
            function legacyError() external;
        }
    }

        #[test]
        fn test_arb_sys_arb_block_number() {
            let v = make_fn_arb_sys_arb_block_number().to_vec();
            let exp = p_arb_sys::arbBlockNumberCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_sys_arb_block_hash(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_sys_arb_block_hash(a0).to_vec();
                let exp = p_arb_sys::arbBlockHashCall { arbBlockNum: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_sys_arb_chain_i_d() {
            let v = make_fn_arb_sys_arb_chain_i_d().to_vec();
            let exp = p_arb_sys::arbChainIDCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_sys_arb_o_s_version() {
            let v = make_fn_arb_sys_arb_o_s_version().to_vec();
            let exp = p_arb_sys::arbOSVersionCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_sys_get_storage_gas_available() {
            let v = make_fn_arb_sys_get_storage_gas_available().to_vec();
            let exp = p_arb_sys::getStorageGasAvailableCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_sys_is_top_level_call() {
            let v = make_fn_arb_sys_is_top_level_call().to_vec();
            let exp = p_arb_sys::isTopLevelCallCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_sys_map_l1_sender_contract_address_to_l2_alias(
                a0 in any::<[u8; 20]>(), a1 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_sys_map_l1_sender_contract_address_to_l2_alias(a0, a1).to_vec();
                let exp = p_arb_sys::mapL1SenderContractAddressToL2AliasCall { sender: AAddress::from(a0), unused: AAddress::from(a1) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_sys_was_my_callers_address_aliased() {
            let v = make_fn_arb_sys_was_my_callers_address_aliased().to_vec();
            let exp = p_arb_sys::wasMyCallersAddressAliasedCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_sys_my_callers_address_without_aliasing() {
            let v = make_fn_arb_sys_my_callers_address_without_aliasing().to_vec();
            let exp = p_arb_sys::myCallersAddressWithoutAliasingCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_sys_withdraw_eth(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_sys_withdraw_eth(a0).to_vec();
                let exp = p_arb_sys::withdrawEthCall { destination: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_sys_send_tx_to_l1_dynamic() {
            let destination = [0x11u8; 20];
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_sys_send_tx_to_l1::<0, 0, 100>(destination, &buf).to_vec();
                let exp = p_arb_sys::sendTxToL1Call { destination: AAddress::from(destination), data: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_sys_send_tx_to_l1::<1, 31, 132>(destination, &buf).to_vec();
                let exp = p_arb_sys::sendTxToL1Call { destination: AAddress::from(destination), data: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_sys_send_tx_to_l1::<32, 0, 132>(destination, &buf).to_vec();
                let exp = p_arb_sys::sendTxToL1Call { destination: AAddress::from(destination), data: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_sys_send_tx_to_l1::<37, 27, 164>(destination, &buf).to_vec();
                let exp = p_arb_sys::sendTxToL1Call { destination: AAddress::from(destination), data: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_sys_send_merkle_tree_state() {
            let v = make_fn_arb_sys_send_merkle_tree_state().to_vec();
            let exp = p_arb_sys::sendMerkleTreeStateCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_info_get_balance(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_info_get_balance(a0).to_vec();
                let exp = p_arb_info::getBalanceCall { account: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_info_get_code(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_info_get_code(a0).to_vec();
                let exp = p_arb_info::getCodeCall { account: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_address_table_address_exists(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_address_table_address_exists(a0).to_vec();
                let exp = p_arb_address_table::addressExistsCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_address_table_compress(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_address_table_compress(a0).to_vec();
                let exp = p_arb_address_table::compressCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_address_table_decompress_dynamic() {
            let offset = U::from([0x22u8; 32]);
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_address_table_decompress::<0, 0, 100>(offset, &buf).to_vec();
                let exp = p_arb_address_table::decompressCall { buf: Bytes::from(buf.to_vec()), offset: AU::from_be_bytes(offset.0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_address_table_decompress::<1, 31, 132>(offset, &buf).to_vec();
                let exp = p_arb_address_table::decompressCall { buf: Bytes::from(buf.to_vec()), offset: AU::from_be_bytes(offset.0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_address_table_decompress::<32, 0, 132>(offset, &buf).to_vec();
                let exp = p_arb_address_table::decompressCall { buf: Bytes::from(buf.to_vec()), offset: AU::from_be_bytes(offset.0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_address_table_decompress::<37, 27, 164>(offset, &buf).to_vec();
                let exp = p_arb_address_table::decompressCall { buf: Bytes::from(buf.to_vec()), offset: AU::from_be_bytes(offset.0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_address_table_lookup(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_address_table_lookup(a0).to_vec();
                let exp = p_arb_address_table::lookupCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_address_table_lookup_index(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_address_table_lookup_index(a0).to_vec();
                let exp = p_arb_address_table::lookupIndexCall { index: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_address_table_register(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_address_table_register(a0).to_vec();
                let exp = p_arb_address_table::registerCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_address_table_size() {
            let v = make_fn_arb_address_table_size().to_vec();
            let exp = p_arb_address_table::sizeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_function_table_upload_dynamic() {
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_function_table_upload::<0, 0, 68>(&buf).to_vec();
                let exp = p_arb_function_table::uploadCall { buf: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_function_table_upload::<1, 31, 100>(&buf).to_vec();
                let exp = p_arb_function_table::uploadCall { buf: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_function_table_upload::<32, 0, 100>(&buf).to_vec();
                let exp = p_arb_function_table::uploadCall { buf: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_function_table_upload::<37, 27, 132>(&buf).to_vec();
                let exp = p_arb_function_table::uploadCall { buf: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_function_table_size(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_function_table_size(a0).to_vec();
                let exp = p_arb_function_table::sizeCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_function_table_get(
                a0 in any::<[u8; 20]>(), a1 in any::<U>()
            ) {
                let v = make_fn_arb_function_table_get(a0, a1).to_vec();
                let exp = p_arb_function_table::getCall { addr: AAddress::from(a0), index: AU::from_be_bytes(*a1) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arbos_test_burn_arb_gas(
                a0 in any::<U>()
            ) {
                let v = make_fn_arbos_test_burn_arb_gas(a0).to_vec();
                let exp = p_arbos_test::burnArbGasCall { gasAmount: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_public_is_chain_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_public_is_chain_owner(a0).to_vec();
                let exp = p_arb_owner_public::isChainOwnerCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_public_rectify_chain_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_public_rectify_chain_owner(a0).to_vec();
                let exp = p_arb_owner_public::rectifyChainOwnerCall { ownerToRectify: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_public_get_all_chain_owners() {
            let v = make_fn_arb_owner_public_get_all_chain_owners().to_vec();
            let exp = p_arb_owner_public::getAllChainOwnersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_native_token_management_from() {
            let v = make_fn_arb_owner_public_get_native_token_management_from().to_vec();
            let exp = p_arb_owner_public::getNativeTokenManagementFromCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_public_is_native_token_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_public_is_native_token_owner(a0).to_vec();
                let exp = p_arb_owner_public::isNativeTokenOwnerCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_public_get_all_native_token_owners() {
            let v = make_fn_arb_owner_public_get_all_native_token_owners().to_vec();
            let exp = p_arb_owner_public::getAllNativeTokenOwnersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_transaction_filtering_from() {
            let v = make_fn_arb_owner_public_get_transaction_filtering_from().to_vec();
            let exp = p_arb_owner_public::getTransactionFilteringFromCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_public_is_transaction_filterer(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_public_is_transaction_filterer(a0).to_vec();
                let exp = p_arb_owner_public::isTransactionFiltererCall { filterer: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_public_get_all_transaction_filterers() {
            let v = make_fn_arb_owner_public_get_all_transaction_filterers().to_vec();
            let exp = p_arb_owner_public::getAllTransactionFilterersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_filtered_funds_recipient() {
            let v = make_fn_arb_owner_public_get_filtered_funds_recipient().to_vec();
            let exp = p_arb_owner_public::getFilteredFundsRecipientCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_network_fee_account() {
            let v = make_fn_arb_owner_public_get_network_fee_account().to_vec();
            let exp = p_arb_owner_public::getNetworkFeeAccountCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_infra_fee_account() {
            let v = make_fn_arb_owner_public_get_infra_fee_account().to_vec();
            let exp = p_arb_owner_public::getInfraFeeAccountCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_brotli_compression_level() {
            let v = make_fn_arb_owner_public_get_brotli_compression_level().to_vec();
            let exp = p_arb_owner_public::getBrotliCompressionLevelCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_parent_gas_floor_per_token() {
            let v = make_fn_arb_owner_public_get_parent_gas_floor_per_token().to_vec();
            let exp = p_arb_owner_public::getParentGasFloorPerTokenCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_scheduled_upgrade() {
            let v = make_fn_arb_owner_public_get_scheduled_upgrade().to_vec();
            let exp = p_arb_owner_public::getScheduledUpgradeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_is_calldata_price_increase_enabled() {
            let v = make_fn_arb_owner_public_is_calldata_price_increase_enabled().to_vec();
            let exp = p_arb_owner_public::isCalldataPriceIncreaseEnabledCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_collect_tips() {
            let v = make_fn_arb_owner_public_get_collect_tips().to_vec();
            let exp = p_arb_owner_public::getCollectTipsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_public_get_max_stylus_contract_fragments() {
            let v = make_fn_arb_owner_public_get_max_stylus_contract_fragments().to_vec();
            let exp = p_arb_owner_public::getMaxStylusContractFragmentsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_gas_info_get_prices_in_wei_with_aggregator(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_gas_info_get_prices_in_wei_with_aggregator(a0).to_vec();
                let exp = p_arb_gas_info::getPricesInWeiWithAggregatorCall { aggregator: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_gas_info_get_prices_in_wei() {
            let v = make_fn_arb_gas_info_get_prices_in_wei().to_vec();
            let exp = p_arb_gas_info::getPricesInWeiCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_gas_info_get_prices_in_arb_gas_with_aggregator(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_gas_info_get_prices_in_arb_gas_with_aggregator(a0).to_vec();
                let exp = p_arb_gas_info::getPricesInArbGasWithAggregatorCall { aggregator: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_gas_info_get_prices_in_arb_gas() {
            let v = make_fn_arb_gas_info_get_prices_in_arb_gas().to_vec();
            let exp = p_arb_gas_info::getPricesInArbGasCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_gas_accounting_params() {
            let v = make_fn_arb_gas_info_get_gas_accounting_params().to_vec();
            let exp = p_arb_gas_info::getGasAccountingParamsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_max_tx_gas_limit() {
            let v = make_fn_arb_gas_info_get_max_tx_gas_limit().to_vec();
            let exp = p_arb_gas_info::getMaxTxGasLimitCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_minimum_gas_price() {
            let v = make_fn_arb_gas_info_get_minimum_gas_price().to_vec();
            let exp = p_arb_gas_info::getMinimumGasPriceCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_base_fee_estimate() {
            let v = make_fn_arb_gas_info_get_l1_base_fee_estimate().to_vec();
            let exp = p_arb_gas_info::getL1BaseFeeEstimateCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_base_fee_estimate_inertia() {
            let v = make_fn_arb_gas_info_get_l1_base_fee_estimate_inertia().to_vec();
            let exp = p_arb_gas_info::getL1BaseFeeEstimateInertiaCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_reward_rate() {
            let v = make_fn_arb_gas_info_get_l1_reward_rate().to_vec();
            let exp = p_arb_gas_info::getL1RewardRateCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_reward_recipient() {
            let v = make_fn_arb_gas_info_get_l1_reward_recipient().to_vec();
            let exp = p_arb_gas_info::getL1RewardRecipientCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_gas_price_estimate() {
            let v = make_fn_arb_gas_info_get_l1_gas_price_estimate().to_vec();
            let exp = p_arb_gas_info::getL1GasPriceEstimateCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_current_tx_l1_gas_fees() {
            let v = make_fn_arb_gas_info_get_current_tx_l1_gas_fees().to_vec();
            let exp = p_arb_gas_info::getCurrentTxL1GasFeesCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_gas_backlog() {
            let v = make_fn_arb_gas_info_get_gas_backlog().to_vec();
            let exp = p_arb_gas_info::getGasBacklogCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_pricing_inertia() {
            let v = make_fn_arb_gas_info_get_pricing_inertia().to_vec();
            let exp = p_arb_gas_info::getPricingInertiaCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_gas_backlog_tolerance() {
            let v = make_fn_arb_gas_info_get_gas_backlog_tolerance().to_vec();
            let exp = p_arb_gas_info::getGasBacklogToleranceCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_pricing_surplus() {
            let v = make_fn_arb_gas_info_get_l1_pricing_surplus().to_vec();
            let exp = p_arb_gas_info::getL1PricingSurplusCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_per_batch_gas_charge() {
            let v = make_fn_arb_gas_info_get_per_batch_gas_charge().to_vec();
            let exp = p_arb_gas_info::getPerBatchGasChargeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_amortized_cost_cap_bips() {
            let v = make_fn_arb_gas_info_get_amortized_cost_cap_bips().to_vec();
            let exp = p_arb_gas_info::getAmortizedCostCapBipsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_fees_available() {
            let v = make_fn_arb_gas_info_get_l1_fees_available().to_vec();
            let exp = p_arb_gas_info::getL1FeesAvailableCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_pricing_equilibration_units() {
            let v = make_fn_arb_gas_info_get_l1_pricing_equilibration_units().to_vec();
            let exp = p_arb_gas_info::getL1PricingEquilibrationUnitsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_last_l1_pricing_update_time() {
            let v = make_fn_arb_gas_info_get_last_l1_pricing_update_time().to_vec();
            let exp = p_arb_gas_info::getLastL1PricingUpdateTimeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_pricing_funds_due_for_rewards() {
            let v = make_fn_arb_gas_info_get_l1_pricing_funds_due_for_rewards().to_vec();
            let exp = p_arb_gas_info::getL1PricingFundsDueForRewardsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_l1_pricing_units_since_update() {
            let v = make_fn_arb_gas_info_get_l1_pricing_units_since_update().to_vec();
            let exp = p_arb_gas_info::getL1PricingUnitsSinceUpdateCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_last_l1_pricing_surplus() {
            let v = make_fn_arb_gas_info_get_last_l1_pricing_surplus().to_vec();
            let exp = p_arb_gas_info::getLastL1PricingSurplusCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_max_block_gas_limit() {
            let v = make_fn_arb_gas_info_get_max_block_gas_limit().to_vec();
            let exp = p_arb_gas_info::getMaxBlockGasLimitCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_gas_pricing_constraints() {
            let v = make_fn_arb_gas_info_get_gas_pricing_constraints().to_vec();
            let exp = p_arb_gas_info::getGasPricingConstraintsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_multi_gas_pricing_constraints() {
            let v = make_fn_arb_gas_info_get_multi_gas_pricing_constraints().to_vec();
            let exp = p_arb_gas_info::getMultiGasPricingConstraintsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_gas_info_get_multi_gas_base_fee() {
            let v = make_fn_arb_gas_info_get_multi_gas_base_fee().to_vec();
            let exp = p_arb_gas_info::getMultiGasBaseFeeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_get_preferred_aggregator(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_aggregator_get_preferred_aggregator(a0).to_vec();
                let exp = p_arb_aggregator::getPreferredAggregatorCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_aggregator_get_default_aggregator() {
            let v = make_fn_arb_aggregator_get_default_aggregator().to_vec();
            let exp = p_arb_aggregator::getDefaultAggregatorCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_aggregator_get_batch_posters() {
            let v = make_fn_arb_aggregator_get_batch_posters().to_vec();
            let exp = p_arb_aggregator::getBatchPostersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_add_batch_poster(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_aggregator_add_batch_poster(a0).to_vec();
                let exp = p_arb_aggregator::addBatchPosterCall { newBatchPoster: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_get_fee_collector(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_aggregator_get_fee_collector(a0).to_vec();
                let exp = p_arb_aggregator::getFeeCollectorCall { batchPoster: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_set_fee_collector(
                a0 in any::<[u8; 20]>(), a1 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_aggregator_set_fee_collector(a0, a1).to_vec();
                let exp = p_arb_aggregator::setFeeCollectorCall { batchPoster: AAddress::from(a0), newFeeCollector: AAddress::from(a1) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_get_tx_base_fee(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_aggregator_get_tx_base_fee(a0).to_vec();
                let exp = p_arb_aggregator::getTxBaseFeeCall { aggregator: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_aggregator_set_tx_base_fee(
                a0 in any::<[u8; 20]>(), a1 in any::<U>()
            ) {
                let v = make_fn_arb_aggregator_set_tx_base_fee(a0, a1).to_vec();
                let exp = p_arb_aggregator::setTxBaseFeeCall { aggregator: AAddress::from(a0), feeInL1Gas: AU::from_be_bytes(*a1) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_retryable_tx_redeem(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_retryable_tx_redeem(a0).to_vec();
                let exp = p_arb_retryable_tx::redeemCall { ticketId: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_retryable_tx_get_lifetime() {
            let v = make_fn_arb_retryable_tx_get_lifetime().to_vec();
            let exp = p_arb_retryable_tx::getLifetimeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_retryable_tx_get_timeout(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_retryable_tx_get_timeout(a0).to_vec();
                let exp = p_arb_retryable_tx::getTimeoutCall { ticketId: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_retryable_tx_keepalive(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_retryable_tx_keepalive(a0).to_vec();
                let exp = p_arb_retryable_tx::keepaliveCall { ticketId: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_retryable_tx_get_beneficiary(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_retryable_tx_get_beneficiary(a0).to_vec();
                let exp = p_arb_retryable_tx::getBeneficiaryCall { ticketId: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_retryable_tx_cancel(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_retryable_tx_cancel(a0).to_vec();
                let exp = p_arb_retryable_tx::cancelCall { ticketId: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_retryable_tx_get_current_redeemer() {
            let v = make_fn_arb_retryable_tx_get_current_redeemer().to_vec();
            let exp = p_arb_retryable_tx::getCurrentRedeemerCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_retryable_tx_submit_retryable_dynamic() {
            let requestId = [0x22u8; 32];
            let l1BaseFee = U::from([0x22u8; 32]);
            let deposit = U::from([0x22u8; 32]);
            let callvalue = U::from([0x22u8; 32]);
            let gasFeeCap = U::from([0x22u8; 32]);
            let gasLimit = 0x33445566778899u64;
            let maxSubmissionFee = U::from([0x22u8; 32]);
            let feeRefundAddress = [0x11u8; 20];
            let beneficiary = [0x11u8; 20];
            let retryTo = [0x11u8; 20];
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_retryable_tx_submit_retryable::<0, 0, 388>(requestId, l1BaseFee, deposit, callvalue, gasFeeCap, gasLimit, maxSubmissionFee, feeRefundAddress, beneficiary, retryTo, &buf).to_vec();
                let exp = p_arb_retryable_tx::submitRetryableCall { requestId: FixedBytes::from(requestId), l1BaseFee: AU::from_be_bytes(l1BaseFee.0), deposit: AU::from_be_bytes(deposit.0), callvalue: AU::from_be_bytes(callvalue.0), gasFeeCap: AU::from_be_bytes(gasFeeCap.0), gasLimit: gasLimit, maxSubmissionFee: AU::from_be_bytes(maxSubmissionFee.0), feeRefundAddress: AAddress::from(feeRefundAddress), beneficiary: AAddress::from(beneficiary), retryTo: AAddress::from(retryTo), retryData: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_retryable_tx_submit_retryable::<1, 31, 420>(requestId, l1BaseFee, deposit, callvalue, gasFeeCap, gasLimit, maxSubmissionFee, feeRefundAddress, beneficiary, retryTo, &buf).to_vec();
                let exp = p_arb_retryable_tx::submitRetryableCall { requestId: FixedBytes::from(requestId), l1BaseFee: AU::from_be_bytes(l1BaseFee.0), deposit: AU::from_be_bytes(deposit.0), callvalue: AU::from_be_bytes(callvalue.0), gasFeeCap: AU::from_be_bytes(gasFeeCap.0), gasLimit: gasLimit, maxSubmissionFee: AU::from_be_bytes(maxSubmissionFee.0), feeRefundAddress: AAddress::from(feeRefundAddress), beneficiary: AAddress::from(beneficiary), retryTo: AAddress::from(retryTo), retryData: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_retryable_tx_submit_retryable::<32, 0, 420>(requestId, l1BaseFee, deposit, callvalue, gasFeeCap, gasLimit, maxSubmissionFee, feeRefundAddress, beneficiary, retryTo, &buf).to_vec();
                let exp = p_arb_retryable_tx::submitRetryableCall { requestId: FixedBytes::from(requestId), l1BaseFee: AU::from_be_bytes(l1BaseFee.0), deposit: AU::from_be_bytes(deposit.0), callvalue: AU::from_be_bytes(callvalue.0), gasFeeCap: AU::from_be_bytes(gasFeeCap.0), gasLimit: gasLimit, maxSubmissionFee: AU::from_be_bytes(maxSubmissionFee.0), feeRefundAddress: AAddress::from(feeRefundAddress), beneficiary: AAddress::from(beneficiary), retryTo: AAddress::from(retryTo), retryData: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_retryable_tx_submit_retryable::<37, 27, 452>(requestId, l1BaseFee, deposit, callvalue, gasFeeCap, gasLimit, maxSubmissionFee, feeRefundAddress, beneficiary, retryTo, &buf).to_vec();
                let exp = p_arb_retryable_tx::submitRetryableCall { requestId: FixedBytes::from(requestId), l1BaseFee: AU::from_be_bytes(l1BaseFee.0), deposit: AU::from_be_bytes(deposit.0), callvalue: AU::from_be_bytes(callvalue.0), gasFeeCap: AU::from_be_bytes(gasFeeCap.0), gasLimit: gasLimit, maxSubmissionFee: AU::from_be_bytes(maxSubmissionFee.0), feeRefundAddress: AAddress::from(feeRefundAddress), beneficiary: AAddress::from(beneficiary), retryTo: AAddress::from(retryTo), retryData: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_statistics_get_stats() {
            let v = make_fn_arb_statistics_get_stats().to_vec();
            let exp = p_arb_statistics::getStatsCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_add_chain_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_add_chain_owner(a0).to_vec();
                let exp = p_arb_owner::addChainOwnerCall { newOwner: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_remove_chain_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_remove_chain_owner(a0).to_vec();
                let exp = p_arb_owner::removeChainOwnerCall { owner: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_is_chain_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_is_chain_owner(a0).to_vec();
                let exp = p_arb_owner::isChainOwnerCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_get_all_chain_owners() {
            let v = make_fn_arb_owner_get_all_chain_owners().to_vec();
            let exp = p_arb_owner::getAllChainOwnersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_native_token_management_from(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_native_token_management_from(a0).to_vec();
                let exp = p_arb_owner::setNativeTokenManagementFromCall { newTimestamp: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_transaction_filtering_from(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_transaction_filtering_from(a0).to_vec();
                let exp = p_arb_owner::setTransactionFilteringFromCall { newTimestamp: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_add_native_token_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_add_native_token_owner(a0).to_vec();
                let exp = p_arb_owner::addNativeTokenOwnerCall { newOwner: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_remove_native_token_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_remove_native_token_owner(a0).to_vec();
                let exp = p_arb_owner::removeNativeTokenOwnerCall { owner: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_is_native_token_owner(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_is_native_token_owner(a0).to_vec();
                let exp = p_arb_owner::isNativeTokenOwnerCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_get_all_native_token_owners() {
            let v = make_fn_arb_owner_get_all_native_token_owners().to_vec();
            let exp = p_arb_owner::getAllNativeTokenOwnersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_add_transaction_filterer(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_add_transaction_filterer(a0).to_vec();
                let exp = p_arb_owner::addTransactionFiltererCall { newFilterer: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_remove_transaction_filterer(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_remove_transaction_filterer(a0).to_vec();
                let exp = p_arb_owner::removeTransactionFiltererCall { filterer: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_is_transaction_filterer(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_is_transaction_filterer(a0).to_vec();
                let exp = p_arb_owner::isTransactionFiltererCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_get_all_transaction_filterers() {
            let v = make_fn_arb_owner_get_all_transaction_filterers().to_vec();
            let exp = p_arb_owner::getAllTransactionFilterersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_filtered_funds_recipient(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_set_filtered_funds_recipient(a0).to_vec();
                let exp = p_arb_owner::setFilteredFundsRecipientCall { receiver: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_get_filtered_funds_recipient() {
            let v = make_fn_arb_owner_get_filtered_funds_recipient().to_vec();
            let exp = p_arb_owner::getFilteredFundsRecipientCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_base_fee_estimate_inertia(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_l1_base_fee_estimate_inertia(a0).to_vec();
                let exp = p_arb_owner::setL1BaseFeeEstimateInertiaCall { inertia: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l2_base_fee(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_owner_set_l2_base_fee(a0).to_vec();
                let exp = p_arb_owner::setL2BaseFeeCall { priceInWei: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_minimum_l2_base_fee(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_owner_set_minimum_l2_base_fee(a0).to_vec();
                let exp = p_arb_owner::setMinimumL2BaseFeeCall { priceInWei: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_speed_limit(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_speed_limit(a0).to_vec();
                let exp = p_arb_owner::setSpeedLimitCall { speedLimit: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_max_tx_gas_limit(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_max_tx_gas_limit(a0).to_vec();
                let exp = p_arb_owner::setMaxTxGasLimitCall { maxTxGasLimit: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_max_block_gas_limit(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_max_block_gas_limit(a0).to_vec();
                let exp = p_arb_owner::setMaxBlockGasLimitCall { maxBlockGasLimit: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l2_gas_pricing_inertia(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_l2_gas_pricing_inertia(a0).to_vec();
                let exp = p_arb_owner::setL2GasPricingInertiaCall { inertia: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l2_gas_backlog_tolerance(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_l2_gas_backlog_tolerance(a0).to_vec();
                let exp = p_arb_owner::setL2GasBacklogToleranceCall { tolerance: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_get_network_fee_account() {
            let v = make_fn_arb_owner_get_network_fee_account().to_vec();
            let exp = p_arb_owner::getNetworkFeeAccountCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_owner_get_infra_fee_account() {
            let v = make_fn_arb_owner_get_infra_fee_account().to_vec();
            let exp = p_arb_owner::getInfraFeeAccountCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_network_fee_account(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_set_network_fee_account(a0).to_vec();
                let exp = p_arb_owner::setNetworkFeeAccountCall { newNetworkFeeAccount: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_infra_fee_account(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_set_infra_fee_account(a0).to_vec();
                let exp = p_arb_owner::setInfraFeeAccountCall { newInfraFeeAccount: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_schedule_arb_o_s_upgrade(
                a0 in any::<u64>(), a1 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_schedule_arb_o_s_upgrade(a0, a1).to_vec();
                let exp = p_arb_owner::scheduleArbOSUpgradeCall { newVersion: a0, timestamp: a1 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_pricing_equilibration_units(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_owner_set_l1_pricing_equilibration_units(a0).to_vec();
                let exp = p_arb_owner::setL1PricingEquilibrationUnitsCall { units: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_pricing_inertia(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_l1_pricing_inertia(a0).to_vec();
                let exp = p_arb_owner::setL1PricingInertiaCall { inertia: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_pricing_reward_recipient(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_set_l1_pricing_reward_recipient(a0).to_vec();
                let exp = p_arb_owner::setL1PricingRewardRecipientCall { recipient: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_pricing_reward_rate(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_l1_pricing_reward_rate(a0).to_vec();
                let exp = p_arb_owner::setL1PricingRewardRateCall { recipientRate: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_l1_price_per_unit(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_owner_set_l1_price_per_unit(a0).to_vec();
                let exp = p_arb_owner::setL1PricePerUnitCall { pricePerUnit: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_parent_gas_floor_per_token(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_parent_gas_floor_per_token(a0).to_vec();
                let exp = p_arb_owner::setParentGasFloorPerTokenCall { floorPerToken: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_per_batch_gas_charge(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_per_batch_gas_charge(a0).to_vec();
                let exp = p_arb_owner::setPerBatchGasChargeCall { charge: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_brotli_compression_level(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_brotli_compression_level(a0).to_vec();
                let exp = p_arb_owner::setBrotliCompressionLevelCall { level: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_amortized_cost_cap_bips(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_amortized_cost_cap_bips(a0).to_vec();
                let exp = p_arb_owner::setAmortizedCostCapBipsCall { capBips: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_release_l1_pricer_surplus_funds(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_release_l1_pricer_surplus_funds(a0).to_vec();
                let exp = p_arb_owner::releaseL1PricerSurplusFundsCall { surplusToRelease: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_ink_price(
                a0 in any::<u32>()
            ) {
                let v = make_fn_arb_owner_set_ink_price(a0).to_vec();
                let exp = p_arb_owner::setInkPriceCall { price: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_max_stack_depth(
                a0 in any::<u32>()
            ) {
                let v = make_fn_arb_owner_set_wasm_max_stack_depth(a0).to_vec();
                let exp = p_arb_owner::setWasmMaxStackDepthCall { depth: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_free_pages(
                a0 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_free_pages(a0).to_vec();
                let exp = p_arb_owner::setWasmFreePagesCall { amount: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_page_gas(
                a0 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_page_gas(a0).to_vec();
                let exp = p_arb_owner::setWasmPageGasCall { amount: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_page_limit(
                a0 in any::<u32>()
            ) {
                let v = make_fn_arb_owner_set_wasm_page_limit(a0).to_vec();
                let exp = p_arb_owner::setWasmPageLimitCall { limit: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_max_size(
                a0 in any::<u32>()
            ) {
                let v = make_fn_arb_owner_set_wasm_max_size(a0).to_vec();
                let exp = p_arb_owner::setWasmMaxSizeCall { size: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_min_init_gas(
                a0 in any::<u8>(), a1 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_min_init_gas(a0, a1).to_vec();
                let exp = p_arb_owner::setWasmMinInitGasCall { gas: a0, cached: a1 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_init_cost_scalar(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_wasm_init_cost_scalar(a0).to_vec();
                let exp = p_arb_owner::setWasmInitCostScalarCall { percent: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_expiry_days(
                a0 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_expiry_days(a0).to_vec();
                let exp = p_arb_owner::setWasmExpiryDaysCall { days: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_keepalive_days(
                a0 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_keepalive_days(a0).to_vec();
                let exp = p_arb_owner::setWasmKeepaliveDaysCall { days: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_block_cache_size(
                a0 in any::<u16>()
            ) {
                let v = make_fn_arb_owner_set_wasm_block_cache_size(a0).to_vec();
                let exp = p_arb_owner::setWasmBlockCacheSizeCall { size: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_add_wasm_cache_manager(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_add_wasm_cache_manager(a0).to_vec();
                let exp = p_arb_owner::addWasmCacheManagerCall { manager: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_remove_wasm_cache_manager(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_owner_remove_wasm_cache_manager(a0).to_vec();
                let exp = p_arb_owner::removeWasmCacheManagerCall { manager: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_set_chain_config_dynamic() {
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_owner_set_chain_config::<0, 0, 68>(&buf).to_vec();
                let exp = p_arb_owner::setChainConfigCall { chainConfig: unsafe { String::from_utf8_unchecked(buf.to_vec()) } }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_owner_set_chain_config::<1, 31, 100>(&buf).to_vec();
                let exp = p_arb_owner::setChainConfigCall { chainConfig: unsafe { String::from_utf8_unchecked(buf.to_vec()) } }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_owner_set_chain_config::<32, 0, 100>(&buf).to_vec();
                let exp = p_arb_owner::setChainConfigCall { chainConfig: unsafe { String::from_utf8_unchecked(buf.to_vec()) } }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_owner_set_chain_config::<37, 27, 132>(&buf).to_vec();
                let exp = p_arb_owner::setChainConfigCall { chainConfig: unsafe { String::from_utf8_unchecked(buf.to_vec()) } }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_calldata_price_increase(
                a0 in any::<bool>()
            ) {
                let v = make_fn_arb_owner_set_calldata_price_increase(a0).to_vec();
                let exp = p_arb_owner::setCalldataPriceIncreaseCall { enable: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_gas_backlog(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_gas_backlog(a0).to_vec();
                let exp = p_arb_owner::setGasBacklogCall { backlog: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_owner_set_gas_pricing_constraints_tail() {
            let exp = p_arb_owner::setGasPricingConstraintsCall {
                constraints: vec![[1u64, 2, 3]],
            }.abi_encode();
            assert_eq!(&exp[..4], &SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS[..], "selector mismatch");
            { // size 0
                let tail = [0xabu8; 0];
                let v = make_fn_arb_owner_set_gas_pricing_constraints::<0, 4>(&tail).to_vec();
                assert_eq!(v.len(), 4);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
            { // size 33
                let tail = [0xabu8; 33];
                let v = make_fn_arb_owner_set_gas_pricing_constraints::<33, 37>(&tail).to_vec();
                assert_eq!(v.len(), 37);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
            { // size 96
                let tail = [0xabu8; 96];
                let v = make_fn_arb_owner_set_gas_pricing_constraints::<96, 100>(&tail).to_vec();
                assert_eq!(v.len(), 100);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
        }

        #[test]
        fn test_arb_owner_set_multi_gas_pricing_constraints_tail() {
            let exp = p_arb_owner::setMultiGasPricingConstraintsCall {
                constraints: vec![p_arb_owner::ResourceConstraint {
                    resources: vec![
                        p_arb_owner::WeightedResource { resource: 1u8, weight: 2u64 },
                        p_arb_owner::WeightedResource { resource: 3u8, weight: 4u64 },
                    ],
                    adjustmentWindowSecs: 5u32,
                    targetPerSec: 6u64,
                    backlog: 7u64,
                }],
            }.abi_encode();
            assert_eq!(&exp[..4], &SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS[..], "selector mismatch");
            { // size 0
                let tail = [0xabu8; 0];
                let v = make_fn_arb_owner_set_multi_gas_pricing_constraints::<0, 4>(&tail).to_vec();
                assert_eq!(v.len(), 4);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
            { // size 33
                let tail = [0xabu8; 33];
                let v = make_fn_arb_owner_set_multi_gas_pricing_constraints::<33, 37>(&tail).to_vec();
                assert_eq!(v.len(), 37);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
            { // size 96
                let tail = [0xabu8; 96];
                let v = make_fn_arb_owner_set_multi_gas_pricing_constraints::<96, 100>(&tail).to_vec();
                assert_eq!(v.len(), 100);
                assert_eq!(&v[..4], &SEL_ARB_OWNER_SETMULTIGASPRICINGCONSTRAINTS[..]);
                assert_eq!(&v[4..], &tail[..]);
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_collect_tips(
                a0 in any::<bool>()
            ) {
                let v = make_fn_arb_owner_set_collect_tips(a0).to_vec();
                let exp = p_arb_owner::setCollectTipsCall { collectTips: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_max_stylus_contract_fragments(
                a0 in any::<u8>()
            ) {
                let v = make_fn_arb_owner_set_max_stylus_contract_fragments(a0).to_vec();
                let exp = p_arb_owner::setMaxStylusContractFragmentsCall { maxFragments: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_owner_set_wasm_activation_gas(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_owner_set_wasm_activation_gas(a0).to_vec();
                let exp = p_arb_owner::setWasmActivationGasCall { gas: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_activate_program(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_activate_program(a0).to_vec();
                let exp = p_arb_wasm::activateProgramCall { program: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_wasm_stylus_version() {
            let v = make_fn_arb_wasm_stylus_version().to_vec();
            let exp = p_arb_wasm::stylusVersionCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_wasm_codehash_version(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_codehash_version(a0).to_vec();
                let exp = p_arb_wasm::codehashVersionCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_codehash_keepalive(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_codehash_keepalive(a0).to_vec();
                let exp = p_arb_wasm::codehashKeepaliveCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_codehash_asm_size(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_codehash_asm_size(a0).to_vec();
                let exp = p_arb_wasm::codehashAsmSizeCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_program_version(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_program_version(a0).to_vec();
                let exp = p_arb_wasm::programVersionCall { program: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_program_init_gas(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_program_init_gas(a0).to_vec();
                let exp = p_arb_wasm::programInitGasCall { program: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_program_memory_footprint(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_program_memory_footprint(a0).to_vec();
                let exp = p_arb_wasm::programMemoryFootprintCall { program: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_program_time_left(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_program_time_left(a0).to_vec();
                let exp = p_arb_wasm::programTimeLeftCall { program: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_wasm_ink_price() {
            let v = make_fn_arb_wasm_ink_price().to_vec();
            let exp = p_arb_wasm::inkPriceCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_max_stack_depth() {
            let v = make_fn_arb_wasm_max_stack_depth().to_vec();
            let exp = p_arb_wasm::maxStackDepthCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_free_pages() {
            let v = make_fn_arb_wasm_free_pages().to_vec();
            let exp = p_arb_wasm::freePagesCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_page_gas() {
            let v = make_fn_arb_wasm_page_gas().to_vec();
            let exp = p_arb_wasm::pageGasCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_page_ramp() {
            let v = make_fn_arb_wasm_page_ramp().to_vec();
            let exp = p_arb_wasm::pageRampCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_page_limit() {
            let v = make_fn_arb_wasm_page_limit().to_vec();
            let exp = p_arb_wasm::pageLimitCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_min_init_gas() {
            let v = make_fn_arb_wasm_min_init_gas().to_vec();
            let exp = p_arb_wasm::minInitGasCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_init_cost_scalar() {
            let v = make_fn_arb_wasm_init_cost_scalar().to_vec();
            let exp = p_arb_wasm::initCostScalarCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_expiry_days() {
            let v = make_fn_arb_wasm_expiry_days().to_vec();
            let exp = p_arb_wasm::expiryDaysCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_keepalive_days() {
            let v = make_fn_arb_wasm_keepalive_days().to_vec();
            let exp = p_arb_wasm::keepaliveDaysCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_block_cache_size() {
            let v = make_fn_arb_wasm_block_cache_size().to_vec();
            let exp = p_arb_wasm::blockCacheSizeCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_wasm_activation_gas() {
            let v = make_fn_arb_wasm_activation_gas().to_vec();
            let exp = p_arb_wasm::activationGasCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_wasm_cache_is_cache_manager(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_cache_is_cache_manager(a0).to_vec();
                let exp = p_arb_wasm_cache::isCacheManagerCall { manager: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_wasm_cache_all_cache_managers() {
            let v = make_fn_arb_wasm_cache_all_cache_managers().to_vec();
            let exp = p_arb_wasm_cache::allCacheManagersCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_wasm_cache_cache_codehash(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_cache_cache_codehash(a0).to_vec();
                let exp = p_arb_wasm_cache::cacheCodehashCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_cache_cache_program(
                a0 in any::<[u8; 20]>()
            ) {
                let v = make_fn_arb_wasm_cache_cache_program(a0).to_vec();
                let exp = p_arb_wasm_cache::cacheProgramCall { addr: AAddress::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_cache_evict_codehash(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_cache_evict_codehash(a0).to_vec();
                let exp = p_arb_wasm_cache::evictCodehashCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_wasm_cache_codehash_is_cached(
                a0 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_wasm_cache_codehash_is_cached(a0).to_vec();
                let exp = p_arb_wasm_cache::codehashIsCachedCall { codehash: FixedBytes::from(a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_native_token_manager_mint_native_token(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_native_token_manager_mint_native_token(a0).to_vec();
                let exp = p_arb_native_token_manager::mintNativeTokenCall { amount: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_native_token_manager_burn_native_token(
                a0 in any::<U>()
            ) {
                let v = make_fn_arb_native_token_manager_burn_native_token(a0).to_vec();
                let exp = p_arb_native_token_manager::burnNativeTokenCall { amount: AU::from_be_bytes(*a0) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_debug_become_chain_owner() {
            let v = make_fn_arb_debug_become_chain_owner().to_vec();
            let exp = p_arb_debug::becomeChainOwnerCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_debug_overwrite_contract_code_dynamic() {
            let target = [0x11u8; 20];
            { // size 0
                let buf = [0xabu8; 0];
                let v = make_fn_arb_debug_overwrite_contract_code::<0, 0, 100>(target, &buf).to_vec();
                let exp = p_arb_debug::overwriteContractCodeCall { target: AAddress::from(target), newCode: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 1
                let buf = [0xabu8; 1];
                let v = make_fn_arb_debug_overwrite_contract_code::<1, 31, 132>(target, &buf).to_vec();
                let exp = p_arb_debug::overwriteContractCodeCall { target: AAddress::from(target), newCode: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 32
                let buf = [0xabu8; 32];
                let v = make_fn_arb_debug_overwrite_contract_code::<32, 0, 132>(target, &buf).to_vec();
                let exp = p_arb_debug::overwriteContractCodeCall { target: AAddress::from(target), newCode: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
            { // size 37
                let buf = [0xabu8; 37];
                let v = make_fn_arb_debug_overwrite_contract_code::<37, 27, 164>(target, &buf).to_vec();
                let exp = p_arb_debug::overwriteContractCodeCall { target: AAddress::from(target), newCode: Bytes::from(buf.to_vec()) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        proptest! {
            #[test]
            fn test_arb_debug_events(
                a0 in any::<bool>(), a1 in any::<[u8; 32]>()
            ) {
                let v = make_fn_arb_debug_events(a0, a1).to_vec();
                let exp = p_arb_debug::eventsCall { flag: a0, value: FixedBytes::from(a1) }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_debug_events_view() {
            let v = make_fn_arb_debug_events_view().to_vec();
            let exp = p_arb_debug::eventsViewCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        proptest! {
            #[test]
            fn test_arb_debug_custom_revert(
                a0 in any::<u64>()
            ) {
                let v = make_fn_arb_debug_custom_revert(a0).to_vec();
                let exp = p_arb_debug::customRevertCall { number: a0 }.abi_encode();
                assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
            }
        }

        #[test]
        fn test_arb_debug_panic() {
            let v = make_fn_arb_debug_panic().to_vec();
            let exp = p_arb_debug::panicCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

        #[test]
        fn test_arb_debug_legacy_error() {
            let v = make_fn_arb_debug_legacy_error().to_vec();
            let exp = p_arb_debug::legacyErrorCall {}.abi_encode();
            assert_eq!(exp, v, "{}{}", const_hex::encode(&exp), const_hex::encode(&v));
        }

}


// ============================================================================


// ============================================================================


// ============================================================================
// Call wrappers (~186) that actually invoke the precompiles. Each pairs with a
// `make_fn_*` calldata builder above. `v:` functions are view/pure and use
// static_call (read-only, no chain mutation); `c:` functions mutate chain state
// and therefore use the real `call` and take `value: &U, gas: u64`.
// ============================================================================

/// Gas limit handed to every read-only (static_call) wrapper.
pub const ARB_CALL_GAS: u64 = u64::MAX;

/// Read one ABI word from `rd[off..off+32]`.
#[inline]
fn word_at<const N: usize>(rd: &[u8; N], off: usize) -> U {
    let mut w = [0u8; 32];
    w.copy_from_slice(&rd[off..off + 32]);
    U::from(w)
}

#[inline]
fn usize_at(v: &[u8], off: usize) -> usize {
    let mut x = 0usize;
    for i in 0..32 { x = x.wrapping_mul(256).wrapping_add(v[off + i] as usize); }
    x
}

#[inline]
fn u64_from(w: U) -> u64 { let b = w.0; let a: [u8; 8] = b[24..32].try_into().unwrap(); u64::from_be_bytes(a) }
#[inline]
fn u32_from(w: U) -> u32 { let b = w.0; let a: [u8; 4] = b[28..32].try_into().unwrap(); u32::from_be_bytes(a) }
#[inline]
fn u16_from(w: U) -> u16 { let b = w.0; let a: [u8; 2] = b[30..32].try_into().unwrap(); u16::from_be_bytes(a) }
#[inline]
fn u8_from(w: U) -> u8 { w.0[31] }
#[inline]
fn addr_from(w: U) -> [u8; 20] { let b = w.0; let a: [u8; 20] = b[12..32].try_into().unwrap(); a }

/// static_call returning a single word, or None on revert (safe: no panic on empty returndata).
#[inline]
fn static_word_opt(addr: [u8; 20], cd: &[u8]) -> Option<U> {
    let (rc, _, rd) = static_call_slice::<32>(addr, cd, ARB_CALL_GAS, 0);
    if rc { Some(U::from(rd)) } else { None }
}

/// static_call returning `n` consecutive fixed words. `CAP` must equal `32 * N` (const arithmetic on
/// generic params is not available on stable Rust, so the caller supplies it as a literal).
fn static_words_opt<const N: usize, const CAP: usize>(addr: [u8; 20], cd: &[u8]) -> Option<[U; N]> {
    let (rc, _, rd) = static_call_slice::<CAP>(addr, cd, ARB_CALL_GAS, 0);
    if !rc { return None; }
    let mut out = [U::ZERO; N];
    for i in 0..N { out[i] = word_at(&rd, i * 32); }
    Some(out)
}

/// call returning a single word, or None on revert.
#[inline]
fn call_word_opt(addr: [u8; 20], cd: &[u8], value: &U, gas: u64) -> Option<U> {
    let (rc, _, rd) = call_slice::<32>(addr, cd, value, gas, 0);
    if rc { Some(U::from(rd)) } else { None }
}

/// call returning `n` consecutive fixed words. `CAP` must equal `32 * N`.
fn call_words_opt<const N: usize, const CAP: usize>(addr: [u8; 20], cd: &[u8], value: &U, gas: u64) -> Option<[U; N]> {
    let (rc, _, rd) = call_slice::<CAP>(addr, cd, value, gas, 0);
    if !rc { return None; }
    let mut out = [U::ZERO; N];
    for i in 0..N { out[i] = word_at(&rd, i * 32); }
    Some(out)
}

#[cfg(feature = "alloc")]
fn decode_addr_vec(v: &[u8]) -> Option<Vec<[u8; 20]>> {
    if v.len() < 32 { return None; }
    let ao = usize_at(v, 0);
    if v.len() < ao + 32 { return None; }
    let n = usize_at(v, ao);
    if v.len() < ao + 32 + 32 * n { return None; }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let s = ao + 32 + 32 * i;
        let a: [u8; 20] = v[s + 12..s + 32].try_into().unwrap();
        out.push(a);
    }
    Some(out)
}

#[cfg(feature = "alloc")]
fn decode_u256_vec(v: &[u8]) -> Option<Vec<U>> {
    if v.len() < 32 { return None; }
    let ao = usize_at(v, 0);
    if v.len() < ao + 32 { return None; }
    let n = usize_at(v, ao);
    if v.len() < ao + 32 + 32 * n { return None; }
    let mut out = Vec::with_capacity(n);
    for i in 0..n { out.push(word_at_vec(v, ao + 32 + 32 * i)); }
    Some(out)
}

#[cfg(feature = "alloc")]
fn word_at_vec(v: &[u8], off: usize) -> U {
    let mut w = [0u8; 32]; w.copy_from_slice(&v[off..off + 32]); U::from(w)
}

#[cfg(feature = "alloc")]
fn decode_triplets(v: &[u8]) -> Option<Vec<[U; 3]>> {
    if v.len() < 32 { return None; }
    let ao = usize_at(v, 0);
    if v.len() < ao + 32 { return None; }
    let n = usize_at(v, ao);
    if v.len() < ao + 32 + 96 * n { return None; }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let s = ao + 32 + 96 * i;
        out.push([word_at_vec(v, s), word_at_vec(v, s + 32), word_at_vec(v, s + 64)]);
    }
    Some(out)
}

#[cfg(feature = "alloc")]
fn decode_merkle(v: &[u8]) -> Option<(U, [u8; 32], Vec<[u8; 32]>)> {
    if v.len() < 96 { return None; }
    let size = word_at_vec(v, 0);
    let root: [u8; 32] = v[32..64].try_into().unwrap();
    let po = usize_at(v, 64);
    if v.len() < po + 32 { return None; }
    let n = usize_at(v, po);
    if v.len() < po + 32 + 32 * n { return None; }
    let mut partials = Vec::with_capacity(n);
    for i in 0..n {
        let s = po + 32 + 32 * i;
        let p: [u8; 32] = v[s..s + 32].try_into().unwrap();
        partials.push(p);
    }
    Some((size, root, partials))
}

#[cfg(feature = "alloc")]
pub struct ConstraintWeighted {
    pub resource: u8,
    pub weight: u64,
}

#[cfg(feature = "alloc")]
pub struct RawResourceConstraint {
    pub resources: alloc::vec::Vec<ConstraintWeighted>,
    pub adjustment_window_secs: u32,
    pub target_per_sec: u64,
    pub backlog: u64,
}

#[cfg(feature = "alloc")]
fn decode_multicons(v: &[u8]) -> Option<Vec<RawResourceConstraint>> {
    if v.len() < 32 { return None; }
    let ao = usize_at(v, 0);
    if v.len() < ao + 32 { return None; }
    let n = usize_at(v, ao);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let s = ao + 32 + 128 * i;
        if v.len() < s + 128 { return None; }
        let adjustment = u32_from(word_at_vec(v, s + 32));
        let target = u64_from(word_at_vec(v, s + 64));
        let backlog = u64_from(word_at_vec(v, s + 96));
        let rp = usize_at(v, s);
        let rr = s + rp;
        if v.len() < rr + 32 { return None; }
        let rn = usize_at(v, rr);
        if v.len() < rr + 32 + 64 * rn { return None; }
        let mut resources = Vec::with_capacity(rn);
        for j in 0..rn {
            let b = rr + 32 + 64 * j;
            let resource = v[b];
            let weight = u64_from(word_at_vec(v, b + 32));
            resources.push(ConstraintWeighted { resource, weight });
        }
        out.push(RawResourceConstraint { resources, adjustment_window_secs: adjustment, target_per_sec: target, backlog });
    }
    Some(out)
}

pub fn call_arb_sys_arb_block_number() -> Option<U> {
    let cd = make_fn_arb_sys_arb_block_number();
    static_word_opt(ADDR_ARB_SYS, &cd)
}

pub fn call_arb_sys_arb_block_hash(arbBlockNum: U) -> Option<U> {
    let cd = make_fn_arb_sys_arb_block_hash(arbBlockNum);
    static_word_opt(ADDR_ARB_SYS, &cd)
}

pub fn call_arb_sys_arb_chain_i_d() -> Option<U> {
    let cd = make_fn_arb_sys_arb_chain_i_d();
    static_word_opt(ADDR_ARB_SYS, &cd)
}

pub fn call_arb_sys_arb_o_s_version() -> Option<U> {
    let cd = make_fn_arb_sys_arb_o_s_version();
    static_word_opt(ADDR_ARB_SYS, &cd)
}

pub fn call_arb_sys_get_storage_gas_available() -> Option<U> {
    let cd = make_fn_arb_sys_get_storage_gas_available();
    static_word_opt(ADDR_ARB_SYS, &cd)
}

pub fn call_arb_sys_is_top_level_call() -> Option<bool> {
    let cd = make_fn_arb_sys_is_top_level_call();
    static_word_opt(ADDR_ARB_SYS, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_sys_map_l1_sender_contract_address_to_l2_alias(sender: [u8; 20], unused: [u8; 20]) -> Option<[u8; 20]> {
    let cd = make_fn_arb_sys_map_l1_sender_contract_address_to_l2_alias(sender, unused);
    static_word_opt(ADDR_ARB_SYS, &cd).map(addr_from)
}

pub fn call_arb_sys_was_my_callers_address_aliased() -> Option<bool> {
    let cd = make_fn_arb_sys_was_my_callers_address_aliased();
    static_word_opt(ADDR_ARB_SYS, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_sys_my_callers_address_without_aliasing() -> Option<[u8; 20]> {
    let cd = make_fn_arb_sys_my_callers_address_without_aliasing();
    static_word_opt(ADDR_ARB_SYS, &cd).map(addr_from)
}

pub fn call_arb_sys_withdraw_eth(destination: [u8; 20], value: &U, gas: u64) -> Option<U> {
    let cd = make_fn_arb_sys_withdraw_eth(destination);
    call_word_opt(ADDR_ARB_SYS, &cd, value, gas)
}

pub fn call_arb_sys_send_tx_to_l1<const N: usize, const PAD: usize, const TOTAL: usize>(destination: [u8; 20], buf: &[u8; N], value: &U, gas: u64) -> Option<U> {
    let cd = make_fn_arb_sys_send_tx_to_l1::<N, PAD, TOTAL>(destination, buf);
    call_word_opt(ADDR_ARB_SYS, &cd, value, gas)
}

#[cfg(feature = "alloc")]
pub fn call_arb_sys_send_merkle_tree_state() -> Option<(U, [u8; 32], Vec<[u8; 32]>)> {
    let cd = make_fn_arb_sys_send_merkle_tree_state();
    let (rc, v) = static_call_vec(ADDR_ARB_SYS, &cd, ARB_CALL_GAS, 0);
    if rc { decode_merkle(&v) } else { None }
}

pub fn call_arb_info_get_balance(account: [u8; 20]) -> Option<U> {
    let cd = make_fn_arb_info_get_balance(account);
    static_word_opt(ADDR_ARB_INFO, &cd)
}

#[cfg(feature = "alloc")]
pub fn call_arb_info_get_code(account: [u8; 20]) -> Option<Vec<u8>> {
    let cd = make_fn_arb_info_get_code(account);
    let (rc, v) = static_call_vec(ADDR_ARB_INFO, &cd, ARB_CALL_GAS, 0);
    if rc { Some(v) } else { None }
}

pub fn call_arb_address_table_address_exists(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_address_table_address_exists(addr);
    static_word_opt(ADDR_ARB_ADDRESS_TABLE, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_address_table_compress(addr: [u8; 20], value: &U, gas: u64) -> Option<Vec<u8>> {
    let cd = make_fn_arb_address_table_compress(addr);
    let (rc, v) = call_vec(ADDR_ARB_ADDRESS_TABLE, &cd, value, gas, 0);
    if rc { Some(v) } else { None }
}

pub fn call_arb_address_table_decompress<const N: usize, const PAD: usize, const TOTAL: usize>(offset: U, buf: &[u8; N]) -> Option<([u8; 20], U)> {
    let cd = make_fn_arb_address_table_decompress::<N, PAD, TOTAL>(offset, buf);
    let ws = static_words_opt::<2, { 32 * 2 }>(ADDR_ARB_ADDRESS_TABLE, &cd)?;
    Some((addr_from(ws[0]), ws[1]))
}

pub fn call_arb_address_table_lookup(addr: [u8; 20]) -> Option<U> {
    let cd = make_fn_arb_address_table_lookup(addr);
    static_word_opt(ADDR_ARB_ADDRESS_TABLE, &cd)
}

pub fn call_arb_address_table_lookup_index(index: U) -> Option<[u8; 20]> {
    let cd = make_fn_arb_address_table_lookup_index(index);
    static_word_opt(ADDR_ARB_ADDRESS_TABLE, &cd).map(addr_from)
}

pub fn call_arb_address_table_register(addr: [u8; 20], value: &U, gas: u64) -> Option<U> {
    let cd = make_fn_arb_address_table_register(addr);
    call_word_opt(ADDR_ARB_ADDRESS_TABLE, &cd, value, gas)
}

pub fn call_arb_address_table_size() -> Option<U> {
    let cd = make_fn_arb_address_table_size();
    static_word_opt(ADDR_ARB_ADDRESS_TABLE, &cd)
}

pub fn call_arb_function_table_upload<const N: usize, const PAD: usize, const TOTAL: usize>(buf: &[u8; N], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_function_table_upload::<N, PAD, TOTAL>(buf);
    call_unit(ADDR_ARB_FUNCTION_TABLE, &cd, value, gas)
}

pub fn call_arb_function_table_size(addr: [u8; 20]) -> Option<U> {
    let cd = make_fn_arb_function_table_size(addr);
    static_word_opt(ADDR_ARB_FUNCTION_TABLE, &cd)
}

pub fn call_arb_function_table_get(addr: [u8; 20], index: U) -> Option<(U, bool, U)> {
    let cd = make_fn_arb_function_table_get(addr, index);
    let ws = static_words_opt::<3, { 32 * 3 }>(ADDR_ARB_FUNCTION_TABLE, &cd)?;
    Some((ws[0], u8_from(ws[1]) != 0, ws[2]))
}

pub fn call_arbos_test_burn_arb_gas(gasAmount: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arbos_test_burn_arb_gas(gasAmount);
    call_unit(ADDR_ARBOS_TEST, &cd, value, gas)
}

pub fn call_arb_owner_public_is_chain_owner(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_public_is_chain_owner(addr);
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_owner_public_rectify_chain_owner(ownerToRectify: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_public_rectify_chain_owner(ownerToRectify);
    call_unit(ADDR_ARB_OWNER_PUBLIC, &cd, value, gas)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_public_get_all_chain_owners() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_public_get_all_chain_owners();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER_PUBLIC, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_public_get_native_token_management_from() -> Option<u64> {
    let cd = make_fn_arb_owner_public_get_native_token_management_from();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u64_from)
}

pub fn call_arb_owner_public_is_native_token_owner(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_public_is_native_token_owner(addr);
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_public_get_all_native_token_owners() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_public_get_all_native_token_owners();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER_PUBLIC, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_public_get_transaction_filtering_from() -> Option<u64> {
    let cd = make_fn_arb_owner_public_get_transaction_filtering_from();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u64_from)
}

pub fn call_arb_owner_public_is_transaction_filterer(filterer: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_public_is_transaction_filterer(filterer);
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_public_get_all_transaction_filterers() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_public_get_all_transaction_filterers();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER_PUBLIC, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_public_get_filtered_funds_recipient() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_public_get_filtered_funds_recipient();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(addr_from)
}

pub fn call_arb_owner_public_get_network_fee_account() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_public_get_network_fee_account();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(addr_from)
}

pub fn call_arb_owner_public_get_infra_fee_account() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_public_get_infra_fee_account();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(addr_from)
}

pub fn call_arb_owner_public_get_brotli_compression_level() -> Option<u64> {
    let cd = make_fn_arb_owner_public_get_brotli_compression_level();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u64_from)
}

pub fn call_arb_owner_public_get_parent_gas_floor_per_token() -> Option<u64> {
    let cd = make_fn_arb_owner_public_get_parent_gas_floor_per_token();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u64_from)
}

pub fn call_arb_owner_public_get_scheduled_upgrade() -> Option<(u64, u64)> {
    let cd = make_fn_arb_owner_public_get_scheduled_upgrade();
    let ws = static_words_opt::<2, { 32 * 2 }>(ADDR_ARB_OWNER_PUBLIC, &cd)?;
    Some((u64_from(ws[0]), u64_from(ws[1])))
}

pub fn call_arb_owner_public_is_calldata_price_increase_enabled() -> Option<bool> {
    let cd = make_fn_arb_owner_public_is_calldata_price_increase_enabled();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_owner_public_get_collect_tips() -> Option<bool> {
    let cd = make_fn_arb_owner_public_get_collect_tips();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_owner_public_get_max_stylus_contract_fragments() -> Option<u8> {
    let cd = make_fn_arb_owner_public_get_max_stylus_contract_fragments();
    static_word_opt(ADDR_ARB_OWNER_PUBLIC, &cd).map(u8_from)
}

pub fn call_arb_gas_info_get_prices_in_wei_with_aggregator(aggregator: [u8; 20]) -> Option<[U; 6]> {
    let cd = make_fn_arb_gas_info_get_prices_in_wei_with_aggregator(aggregator);
    static_words_opt::<6, { 32 * 6 }>(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_prices_in_wei() -> Option<[U; 6]> {
    let cd = make_fn_arb_gas_info_get_prices_in_wei();
    static_words_opt::<6, { 32 * 6 }>(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_prices_in_arb_gas_with_aggregator(aggregator: [u8; 20]) -> Option<[U; 3]> {
    let cd = make_fn_arb_gas_info_get_prices_in_arb_gas_with_aggregator(aggregator);
    static_words_opt::<3, { 32 * 3 }>(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_prices_in_arb_gas() -> Option<[U; 3]> {
    let cd = make_fn_arb_gas_info_get_prices_in_arb_gas();
    static_words_opt::<3, { 32 * 3 }>(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_gas_accounting_params() -> Option<[U; 3]> {
    let cd = make_fn_arb_gas_info_get_gas_accounting_params();
    static_words_opt::<3, { 32 * 3 }>(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_max_tx_gas_limit() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_max_tx_gas_limit();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_minimum_gas_price() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_minimum_gas_price();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_l1_base_fee_estimate() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_base_fee_estimate();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_l1_base_fee_estimate_inertia() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_l1_base_fee_estimate_inertia();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_l1_reward_rate() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_l1_reward_rate();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_l1_reward_recipient() -> Option<[u8; 20]> {
    let cd = make_fn_arb_gas_info_get_l1_reward_recipient();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(addr_from)
}

pub fn call_arb_gas_info_get_l1_gas_price_estimate() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_gas_price_estimate();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_current_tx_l1_gas_fees() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_current_tx_l1_gas_fees();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_gas_backlog() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_gas_backlog();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_pricing_inertia() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_pricing_inertia();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_gas_backlog_tolerance() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_gas_backlog_tolerance();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_l1_pricing_surplus() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_pricing_surplus();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_per_batch_gas_charge() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_per_batch_gas_charge();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_amortized_cost_cap_bips() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_amortized_cost_cap_bips();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_l1_fees_available() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_fees_available();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_l1_pricing_equilibration_units() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_pricing_equilibration_units();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_last_l1_pricing_update_time() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_last_l1_pricing_update_time();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_l1_pricing_funds_due_for_rewards() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_l1_pricing_funds_due_for_rewards();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_l1_pricing_units_since_update() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_l1_pricing_units_since_update();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

pub fn call_arb_gas_info_get_last_l1_pricing_surplus() -> Option<U> {
    let cd = make_fn_arb_gas_info_get_last_l1_pricing_surplus();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd)
}

pub fn call_arb_gas_info_get_max_block_gas_limit() -> Option<u64> {
    let cd = make_fn_arb_gas_info_get_max_block_gas_limit();
    static_word_opt(ADDR_ARB_GAS_INFO, &cd).map(u64_from)
}

#[cfg(feature = "alloc")]
pub fn call_arb_gas_info_get_gas_pricing_constraints() -> Option<Vec<[U; 3]>> {
    let cd = make_fn_arb_gas_info_get_gas_pricing_constraints();
    let (rc, v) = static_call_vec(ADDR_ARB_GAS_INFO, &cd, ARB_CALL_GAS, 0);
    if rc { decode_triplets(&v) } else { None }
}

#[cfg(feature = "alloc")]
pub fn call_arb_gas_info_get_multi_gas_pricing_constraints() -> Option<Vec<RawResourceConstraint>> {
    let cd = make_fn_arb_gas_info_get_multi_gas_pricing_constraints();
    let (rc, v) = static_call_vec(ADDR_ARB_GAS_INFO, &cd, ARB_CALL_GAS, 0);
    if rc { decode_multicons(&v) } else { None }
}

#[cfg(feature = "alloc")]
pub fn call_arb_gas_info_get_multi_gas_base_fee() -> Option<Vec<U>> {
    let cd = make_fn_arb_gas_info_get_multi_gas_base_fee();
    let (rc, v) = static_call_vec(ADDR_ARB_GAS_INFO, &cd, ARB_CALL_GAS, 0);
    if rc { decode_u256_vec(&v) } else { None }
}

pub fn call_arb_aggregator_get_preferred_aggregator(addr: [u8; 20]) -> Option<([u8; 20], bool)> {
    let cd = make_fn_arb_aggregator_get_preferred_aggregator(addr);
    let ws = static_words_opt::<2, { 32 * 2 }>(ADDR_ARB_AGGREGATOR, &cd)?;
    Some((addr_from(ws[0]), u8_from(ws[1]) != 0))
}

pub fn call_arb_aggregator_get_default_aggregator() -> Option<[u8; 20]> {
    let cd = make_fn_arb_aggregator_get_default_aggregator();
    static_word_opt(ADDR_ARB_AGGREGATOR, &cd).map(addr_from)
}

#[cfg(feature = "alloc")]
pub fn call_arb_aggregator_get_batch_posters() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_aggregator_get_batch_posters();
    let (rc, v) = static_call_vec(ADDR_ARB_AGGREGATOR, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_aggregator_add_batch_poster(newBatchPoster: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_aggregator_add_batch_poster(newBatchPoster);
    call_unit(ADDR_ARB_AGGREGATOR, &cd, value, gas)
}

pub fn call_arb_aggregator_get_fee_collector(batchPoster: [u8; 20]) -> Option<[u8; 20]> {
    let cd = make_fn_arb_aggregator_get_fee_collector(batchPoster);
    static_word_opt(ADDR_ARB_AGGREGATOR, &cd).map(addr_from)
}

pub fn call_arb_aggregator_set_fee_collector(batchPoster: [u8; 20], newFeeCollector: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_aggregator_set_fee_collector(batchPoster, newFeeCollector);
    call_unit(ADDR_ARB_AGGREGATOR, &cd, value, gas)
}

pub fn call_arb_aggregator_get_tx_base_fee(aggregator: [u8; 20]) -> Option<U> {
    let cd = make_fn_arb_aggregator_get_tx_base_fee(aggregator);
    static_word_opt(ADDR_ARB_AGGREGATOR, &cd)
}

pub fn call_arb_aggregator_set_tx_base_fee(aggregator: [u8; 20], feeInL1Gas: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_aggregator_set_tx_base_fee(aggregator, feeInL1Gas);
    call_unit(ADDR_ARB_AGGREGATOR, &cd, value, gas)
}

pub fn call_arb_retryable_tx_redeem(ticketId: [u8; 32], value: &U, gas: u64) -> Option<U> {
    let cd = make_fn_arb_retryable_tx_redeem(ticketId);
    call_word_opt(ADDR_ARB_RETRYABLE_TX, &cd, value, gas)
}

pub fn call_arb_retryable_tx_get_lifetime() -> Option<U> {
    let cd = make_fn_arb_retryable_tx_get_lifetime();
    static_word_opt(ADDR_ARB_RETRYABLE_TX, &cd)
}

pub fn call_arb_retryable_tx_get_timeout(ticketId: [u8; 32]) -> Option<U> {
    let cd = make_fn_arb_retryable_tx_get_timeout(ticketId);
    static_word_opt(ADDR_ARB_RETRYABLE_TX, &cd)
}

pub fn call_arb_retryable_tx_keepalive(ticketId: [u8; 32], value: &U, gas: u64) -> Option<U> {
    let cd = make_fn_arb_retryable_tx_keepalive(ticketId);
    call_word_opt(ADDR_ARB_RETRYABLE_TX, &cd, value, gas)
}

pub fn call_arb_retryable_tx_get_beneficiary(ticketId: [u8; 32]) -> Option<[u8; 20]> {
    let cd = make_fn_arb_retryable_tx_get_beneficiary(ticketId);
    static_word_opt(ADDR_ARB_RETRYABLE_TX, &cd).map(addr_from)
}

pub fn call_arb_retryable_tx_cancel(ticketId: [u8; 32], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_retryable_tx_cancel(ticketId);
    call_unit(ADDR_ARB_RETRYABLE_TX, &cd, value, gas)
}

pub fn call_arb_retryable_tx_get_current_redeemer() -> Option<[u8; 20]> {
    let cd = make_fn_arb_retryable_tx_get_current_redeemer();
    static_word_opt(ADDR_ARB_RETRYABLE_TX, &cd).map(addr_from)
}

pub fn call_arb_retryable_tx_submit_retryable<const N: usize, const PAD: usize, const TOTAL: usize>(requestId: [u8; 32], l1BaseFee: U, deposit: U, callvalue: U, gasFeeCap: U, gasLimit: u64, maxSubmissionFee: U, feeRefundAddress: [u8; 20], beneficiary: [u8; 20], retryTo: [u8; 20], buf: &[u8; N], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_retryable_tx_submit_retryable::<N, PAD, TOTAL>(requestId, l1BaseFee, deposit, callvalue, gasFeeCap, gasLimit, maxSubmissionFee, feeRefundAddress, beneficiary, retryTo, buf);
    call_unit(ADDR_ARB_RETRYABLE_TX, &cd, value, gas)
}

pub fn call_arb_statistics_get_stats() -> Option<[U; 6]> {
    let cd = make_fn_arb_statistics_get_stats();
    static_words_opt::<6, { 32 * 6 }>(ADDR_ARB_STATISTICS, &cd)
}

pub fn call_arb_owner_add_chain_owner(newOwner: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_add_chain_owner(newOwner);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_remove_chain_owner(owner: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_remove_chain_owner(owner);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_is_chain_owner(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_is_chain_owner(addr);
    static_word_opt(ADDR_ARB_OWNER, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_get_all_chain_owners() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_get_all_chain_owners();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_set_native_token_management_from(newTimestamp: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_native_token_management_from(newTimestamp);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_transaction_filtering_from(newTimestamp: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_transaction_filtering_from(newTimestamp);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_add_native_token_owner(newOwner: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_add_native_token_owner(newOwner);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_remove_native_token_owner(owner: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_remove_native_token_owner(owner);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_is_native_token_owner(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_is_native_token_owner(addr);
    static_word_opt(ADDR_ARB_OWNER, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_get_all_native_token_owners() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_get_all_native_token_owners();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_add_transaction_filterer(newFilterer: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_add_transaction_filterer(newFilterer);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_remove_transaction_filterer(filterer: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_remove_transaction_filterer(filterer);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_is_transaction_filterer(addr: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_owner_is_transaction_filterer(addr);
    static_word_opt(ADDR_ARB_OWNER, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_owner_get_all_transaction_filterers() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_owner_get_all_transaction_filterers();
    let (rc, v) = static_call_vec(ADDR_ARB_OWNER, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_owner_set_filtered_funds_recipient(receiver: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_filtered_funds_recipient(receiver);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_get_filtered_funds_recipient() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_get_filtered_funds_recipient();
    static_word_opt(ADDR_ARB_OWNER, &cd).map(addr_from)
}

pub fn call_arb_owner_set_l1_base_fee_estimate_inertia(inertia: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_base_fee_estimate_inertia(inertia);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l2_base_fee(priceInWei: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l2_base_fee(priceInWei);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_minimum_l2_base_fee(priceInWei: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_minimum_l2_base_fee(priceInWei);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_speed_limit(speedLimit: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_speed_limit(speedLimit);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_max_tx_gas_limit(maxTxGasLimit: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_max_tx_gas_limit(maxTxGasLimit);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_max_block_gas_limit(maxBlockGasLimit: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_max_block_gas_limit(maxBlockGasLimit);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l2_gas_pricing_inertia(inertia: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l2_gas_pricing_inertia(inertia);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l2_gas_backlog_tolerance(tolerance: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l2_gas_backlog_tolerance(tolerance);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_get_network_fee_account() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_get_network_fee_account();
    static_word_opt(ADDR_ARB_OWNER, &cd).map(addr_from)
}

pub fn call_arb_owner_get_infra_fee_account() -> Option<[u8; 20]> {
    let cd = make_fn_arb_owner_get_infra_fee_account();
    static_word_opt(ADDR_ARB_OWNER, &cd).map(addr_from)
}

pub fn call_arb_owner_set_network_fee_account(newNetworkFeeAccount: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_network_fee_account(newNetworkFeeAccount);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_infra_fee_account(newInfraFeeAccount: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_infra_fee_account(newInfraFeeAccount);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_schedule_arb_o_s_upgrade(newVersion: u64, timestamp: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_schedule_arb_o_s_upgrade(newVersion, timestamp);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l1_pricing_equilibration_units(units: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_pricing_equilibration_units(units);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l1_pricing_inertia(inertia: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_pricing_inertia(inertia);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l1_pricing_reward_recipient(recipient: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_pricing_reward_recipient(recipient);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l1_pricing_reward_rate(recipientRate: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_pricing_reward_rate(recipientRate);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_l1_price_per_unit(pricePerUnit: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_l1_price_per_unit(pricePerUnit);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_parent_gas_floor_per_token(floorPerToken: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_parent_gas_floor_per_token(floorPerToken);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_per_batch_gas_charge(charge: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_per_batch_gas_charge(charge);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_brotli_compression_level(level: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_brotli_compression_level(level);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_amortized_cost_cap_bips(capBips: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_amortized_cost_cap_bips(capBips);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_release_l1_pricer_surplus_funds(surplusToRelease: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_release_l1_pricer_surplus_funds(surplusToRelease);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_ink_price(price: u32, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_ink_price(price);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_max_stack_depth(depth: u32, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_max_stack_depth(depth);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_free_pages(amount: u16, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_free_pages(amount);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_page_gas(amount: u16, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_page_gas(amount);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_page_limit(limit: u32, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_page_limit(limit);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_max_size(size: u32, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_max_size(size);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_min_init_gas(gas: u8, cached: u16, value: &U, call_gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_min_init_gas(gas, cached);
    call_unit(ADDR_ARB_OWNER, &cd, value, call_gas)
}

pub fn call_arb_owner_set_wasm_init_cost_scalar(percent: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_init_cost_scalar(percent);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_expiry_days(days: u16, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_expiry_days(days);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_keepalive_days(days: u16, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_keepalive_days(days);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_block_cache_size(size: u16, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_block_cache_size(size);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_add_wasm_cache_manager(manager: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_add_wasm_cache_manager(manager);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_remove_wasm_cache_manager(manager: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_remove_wasm_cache_manager(manager);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_chain_config<const N: usize, const PAD: usize, const TOTAL: usize>(buf: &[u8; N], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_chain_config::<N, PAD, TOTAL>(buf);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_calldata_price_increase(enable: bool, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_calldata_price_increase(enable);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_gas_backlog(backlog: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_gas_backlog(backlog);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_gas_pricing_constraints<const N: usize, const TOTAL: usize>(tail: &[u8; N], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_gas_pricing_constraints::<N, TOTAL>(tail);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_multi_gas_pricing_constraints<const N: usize, const TOTAL: usize>(tail: &[u8; N], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_multi_gas_pricing_constraints::<N, TOTAL>(tail);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_collect_tips(collectTips: bool, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_collect_tips(collectTips);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_max_stylus_contract_fragments(maxFragments: u8, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_max_stylus_contract_fragments(maxFragments);
    call_unit(ADDR_ARB_OWNER, &cd, value, gas)
}

pub fn call_arb_owner_set_wasm_activation_gas(gas: u64, value: &U, call_gas: u64) -> bool {
    let cd = make_fn_arb_owner_set_wasm_activation_gas(gas);
    call_unit(ADDR_ARB_OWNER, &cd, value, call_gas)
}

pub fn call_arb_wasm_activate_program(program: [u8; 20], value: &U, gas: u64) -> Option<(u16, U)> {
    let cd = make_fn_arb_wasm_activate_program(program);
    let ws = call_words_opt::<2, { 32 * 2 }>(ADDR_ARB_WASM, &cd, value, gas)?;
    Some((u16_from(ws[0]), ws[1]))
}

pub fn call_arb_wasm_stylus_version() -> Option<u16> {
    let cd = make_fn_arb_wasm_stylus_version();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_codehash_version(codehash: [u8; 32]) -> Option<u16> {
    let cd = make_fn_arb_wasm_codehash_version(codehash);
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_codehash_keepalive(codehash: [u8; 32], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_wasm_codehash_keepalive(codehash);
    call_unit(ADDR_ARB_WASM, &cd, value, gas)
}

pub fn call_arb_wasm_codehash_asm_size(codehash: [u8; 32]) -> Option<u32> {
    let cd = make_fn_arb_wasm_codehash_asm_size(codehash);
    static_word_opt(ADDR_ARB_WASM, &cd).map(u32_from)
}

pub fn call_arb_wasm_program_version(program: [u8; 20]) -> Option<u16> {
    let cd = make_fn_arb_wasm_program_version(program);
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_program_init_gas(program: [u8; 20]) -> Option<(u64, u64)> {
    let cd = make_fn_arb_wasm_program_init_gas(program);
    let ws = static_words_opt::<2, { 32 * 2 }>(ADDR_ARB_WASM, &cd)?;
    Some((u64_from(ws[0]), u64_from(ws[1])))
}

pub fn call_arb_wasm_program_memory_footprint(program: [u8; 20]) -> Option<u16> {
    let cd = make_fn_arb_wasm_program_memory_footprint(program);
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_program_time_left(program: [u8; 20]) -> Option<u64> {
    let cd = make_fn_arb_wasm_program_time_left(program);
    static_word_opt(ADDR_ARB_WASM, &cd).map(u64_from)
}

pub fn call_arb_wasm_ink_price() -> Option<u32> {
    let cd = make_fn_arb_wasm_ink_price();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u32_from)
}

pub fn call_arb_wasm_max_stack_depth() -> Option<u32> {
    let cd = make_fn_arb_wasm_max_stack_depth();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u32_from)
}

pub fn call_arb_wasm_free_pages() -> Option<u16> {
    let cd = make_fn_arb_wasm_free_pages();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_page_gas() -> Option<u16> {
    let cd = make_fn_arb_wasm_page_gas();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_page_ramp() -> Option<u64> {
    let cd = make_fn_arb_wasm_page_ramp();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u64_from)
}

pub fn call_arb_wasm_page_limit() -> Option<u16> {
    let cd = make_fn_arb_wasm_page_limit();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_min_init_gas() -> Option<(u64, u64)> {
    let cd = make_fn_arb_wasm_min_init_gas();
    let ws = static_words_opt::<2, { 32 * 2 }>(ADDR_ARB_WASM, &cd)?;
    Some((u64_from(ws[0]), u64_from(ws[1])))
}

pub fn call_arb_wasm_init_cost_scalar() -> Option<u64> {
    let cd = make_fn_arb_wasm_init_cost_scalar();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u64_from)
}

pub fn call_arb_wasm_expiry_days() -> Option<u16> {
    let cd = make_fn_arb_wasm_expiry_days();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_keepalive_days() -> Option<u16> {
    let cd = make_fn_arb_wasm_keepalive_days();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_block_cache_size() -> Option<u16> {
    let cd = make_fn_arb_wasm_block_cache_size();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u16_from)
}

pub fn call_arb_wasm_activation_gas() -> Option<u64> {
    let cd = make_fn_arb_wasm_activation_gas();
    static_word_opt(ADDR_ARB_WASM, &cd).map(u64_from)
}

pub fn call_arb_wasm_cache_is_cache_manager(manager: [u8; 20]) -> Option<bool> {
    let cd = make_fn_arb_wasm_cache_is_cache_manager(manager);
    static_word_opt(ADDR_ARB_WASM_CACHE, &cd).map(u8_from).map(|b| b != 0)
}

#[cfg(feature = "alloc")]
pub fn call_arb_wasm_cache_all_cache_managers() -> Option<Vec<[u8; 20]>> {
    let cd = make_fn_arb_wasm_cache_all_cache_managers();
    let (rc, v) = static_call_vec(ADDR_ARB_WASM_CACHE, &cd, ARB_CALL_GAS, 0);
    if rc { decode_addr_vec(&v) } else { None }
}

pub fn call_arb_wasm_cache_cache_codehash(codehash: [u8; 32], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_wasm_cache_cache_codehash(codehash);
    call_unit(ADDR_ARB_WASM_CACHE, &cd, value, gas)
}

pub fn call_arb_wasm_cache_cache_program(addr: [u8; 20], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_wasm_cache_cache_program(addr);
    call_unit(ADDR_ARB_WASM_CACHE, &cd, value, gas)
}

pub fn call_arb_wasm_cache_evict_codehash(codehash: [u8; 32], value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_wasm_cache_evict_codehash(codehash);
    call_unit(ADDR_ARB_WASM_CACHE, &cd, value, gas)
}

pub fn call_arb_wasm_cache_codehash_is_cached(codehash: [u8; 32]) -> Option<bool> {
    let cd = make_fn_arb_wasm_cache_codehash_is_cached(codehash);
    static_word_opt(ADDR_ARB_WASM_CACHE, &cd).map(u8_from).map(|b| b != 0)
}

pub fn call_arb_native_token_manager_mint_native_token(amount: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_native_token_manager_mint_native_token(amount);
    call_unit(ADDR_ARB_NATIVE_TOKEN_MANAGER, &cd, value, gas)
}

pub fn call_arb_native_token_manager_burn_native_token(amount: U, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_native_token_manager_burn_native_token(amount);
    call_unit(ADDR_ARB_NATIVE_TOKEN_MANAGER, &cd, value, gas)
}

pub fn call_arb_debug_become_chain_owner(value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_debug_become_chain_owner();
    call_unit(ADDR_ARB_DEBUG, &cd, value, gas)
}

#[cfg(feature = "alloc")]
pub fn call_arb_debug_overwrite_contract_code<const N: usize, const PAD: usize, const TOTAL: usize>(target: [u8; 20], buf: &[u8; N], value: &U, gas: u64) -> Option<Vec<u8>> {
    let cd = make_fn_arb_debug_overwrite_contract_code::<N, PAD, TOTAL>(target, buf);
    let (rc, v) = call_vec(ADDR_ARB_DEBUG, &cd, value, gas, 0);
    if rc { Some(v) } else { None }
}

pub fn call_arb_debug_events(flag: bool, value: [u8; 32], call_value: &U, gas: u64) -> Option<([u8; 20], U)> {
    let cd = make_fn_arb_debug_events(flag, value);
    let ws = call_words_opt::<2, { 32 * 2 }>(ADDR_ARB_DEBUG, &cd, call_value, gas)?;
    Some((addr_from(ws[0]), ws[1]))
}

pub fn call_arb_debug_events_view() -> bool {
    let cd = make_fn_arb_debug_events_view();
    static_call_unit(ADDR_ARB_DEBUG, &cd, ARB_CALL_GAS)
}

pub fn call_arb_debug_custom_revert(number: u64, value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_debug_custom_revert(number);
    call_unit(ADDR_ARB_DEBUG, &cd, value, gas)
}

pub fn call_arb_debug_panic(value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_debug_panic();
    call_unit(ADDR_ARB_DEBUG, &cd, value, gas)
}

pub fn call_arb_debug_legacy_error(value: &U, gas: u64) -> bool {
    let cd = make_fn_arb_debug_legacy_error();
    call_unit(ADDR_ARB_DEBUG, &cd, value, gas)
}

