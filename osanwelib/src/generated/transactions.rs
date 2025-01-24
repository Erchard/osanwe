// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionType1 {
    /// 32 байти: Хеш транзакції
    #[prost(bytes = "vec", tag = "1")]
    pub transaction_hash: ::prost::alloc::vec::Vec<u8>,
    /// 4 байти: Тип транзакції (1)
    #[prost(uint32, tag = "2")]
    pub transaction_type: u32,
    /// 4 байти: Криптовалюта (номер зі довідника)
    #[prost(uint32, tag = "3")]
    pub currency_id: u32,
    /// 4 байти: Сума (uint32)
    #[prost(uint32, tag = "4")]
    pub amount: u32,
    /// 8 байтів: Таймстемп з точністю до секунди
    #[prost(uint64, tag = "5")]
    pub timestamp: u64,
    /// 20 байтів: Адреса відправника
    #[prost(bytes = "vec", tag = "6")]
    pub sender_address: ::prost::alloc::vec::Vec<u8>,
    /// 4 байти: Порядковий номер вихідної транзакції відправника
    #[prost(uint32, tag = "7")]
    pub sender_output_index: u32,
    /// 20 байтів: Адреса отримувача
    #[prost(bytes = "vec", tag = "8")]
    pub recipient_address: ::prost::alloc::vec::Vec<u8>,
    /// 65 байтів: Підпис відправника
    #[prost(bytes = "vec", tag = "9")]
    pub sender_signature: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionType2 {
    /// 32 байти: Хеш цієї транзакції
    #[prost(bytes = "vec", tag = "1")]
    pub transaction_hash: ::prost::alloc::vec::Vec<u8>,
    /// 4 байти: Тип транзакції (2)
    #[prost(uint32, tag = "2")]
    pub transaction_type: u32,
    /// 4 байти: Криптовалюта (включно з токенами та блокчейном)
    #[prost(uint32, tag = "3")]
    pub currency_id: u32,
    /// 4 байти: Сума (uint32)
    #[prost(uint32, tag = "4")]
    pub amount: u32,
    /// 32 байти: Хеш транзакції поповнення в блокчейні
    #[prost(bytes = "vec", tag = "5")]
    pub source_transaction_hash: ::prost::alloc::vec::Vec<u8>,
    /// 8 байтів: Таймстемп з точністю до секунди
    #[prost(uint64, tag = "6")]
    pub timestamp: u64,
    /// 20 байтів: Адреса отримувача
    #[prost(bytes = "vec", tag = "7")]
    pub recipient_address: ::prost::alloc::vec::Vec<u8>,
}
