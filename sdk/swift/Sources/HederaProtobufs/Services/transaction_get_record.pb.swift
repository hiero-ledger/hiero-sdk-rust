// DO NOT EDIT.
// swift-format-ignore-file
//
// Generated by the Swift generator plugin for the protocol buffer compiler.
// Source: transaction_get_record.proto
//
// For information on using the generated types, please see the documentation:
//   https://github.com/apple/swift-protobuf/

import Foundation
import SwiftProtobuf

// If the compiler emits an error on this type, it is because this file
// was generated by a version of the `protoc` Swift plug-in that is
// incompatible with the version of SwiftProtobuf to which you are linking.
// Please ensure that you are building against the same version of the API
// that was used to generate this file.
fileprivate struct _GeneratedWithProtocGenSwiftVersion: SwiftProtobuf.ProtobufAPIVersionCheck {
  struct _2: SwiftProtobuf.ProtobufAPIVersion_2 {}
  typealias Version = _2
}

///*
/// Get the record for a transaction. If the transaction requested a record, then the record lasts
/// for one hour, and a state proof is available for it. If the transaction created an account, file,
/// or smart contract instance, then the record will contain the ID for what it created. If the
/// transaction called a smart contract function, then the record contains the result of that call.
/// If the transaction was a cryptocurrency transfer, then the record includes the TransferList which
/// gives the details of that transfer. If the transaction didn't return anything that should be in
/// the record, then the results field will be set to nothing.
public struct Proto_TransactionGetRecordQuery {
  // SwiftProtobuf.Message conformance is added in an extension below. See the
  // `Message` and `Message+*Additions` files in the SwiftProtobuf library for
  // methods supported on all messages.

  ///*
  /// Standard info sent from client to node, including the signed payment, and what kind of
  /// response is requested (cost, state proof, both, or neither).
  public var header: Proto_QueryHeader {
    get {return _header ?? Proto_QueryHeader()}
    set {_header = newValue}
  }
  /// Returns true if `header` has been explicitly set.
  public var hasHeader: Bool {return self._header != nil}
  /// Clears the value of `header`. Subsequent reads from it will return its default value.
  public mutating func clearHeader() {self._header = nil}

  ///*
  /// The ID of the transaction for which the record is requested.
  public var transactionID: Proto_TransactionID {
    get {return _transactionID ?? Proto_TransactionID()}
    set {_transactionID = newValue}
  }
  /// Returns true if `transactionID` has been explicitly set.
  public var hasTransactionID: Bool {return self._transactionID != nil}
  /// Clears the value of `transactionID`. Subsequent reads from it will return its default value.
  public mutating func clearTransactionID() {self._transactionID = nil}

  ///*
  /// Whether records of processing duplicate transactions should be returned along with the record
  /// of processing the first consensus transaction with the given id whose status was neither
  /// <tt>INVALID_NODE_ACCOUNT</tt> nor <tt>INVALID_PAYER_SIGNATURE</tt>; <b>or</b>, if no such
  /// record exists, the record of processing the first transaction to reach consensus with the
  /// given transaction id..
  public var includeDuplicates: Bool = false

  ///*
  /// Whether the response should include the records of any child transactions spawned by the
  /// top-level transaction with the given transactionID.
  public var includeChildRecords: Bool = false

  public var unknownFields = SwiftProtobuf.UnknownStorage()

  public init() {}

  fileprivate var _header: Proto_QueryHeader? = nil
  fileprivate var _transactionID: Proto_TransactionID? = nil
}

///*
/// Response when the client sends the node TransactionGetRecordQuery
public struct Proto_TransactionGetRecordResponse {
  // SwiftProtobuf.Message conformance is added in an extension below. See the
  // `Message` and `Message+*Additions` files in the SwiftProtobuf library for
  // methods supported on all messages.

  ///*
  /// Standard response from node to client, including the requested fields: cost, or state proof,
  /// or both, or neither.
  public var header: Proto_ResponseHeader {
    get {return _header ?? Proto_ResponseHeader()}
    set {_header = newValue}
  }
  /// Returns true if `header` has been explicitly set.
  public var hasHeader: Bool {return self._header != nil}
  /// Clears the value of `header`. Subsequent reads from it will return its default value.
  public mutating func clearHeader() {self._header = nil}

  ///*
  /// Either the record of processing the first consensus transaction with the given id whose
  /// status was neither <tt>INVALID_NODE_ACCOUNT</tt> nor <tt>INVALID_PAYER_SIGNATURE</tt>;
  /// <b>or</b>, if no such record exists, the record of processing the first transaction to reach
  /// consensus with the given transaction id.
  public var transactionRecord: Proto_TransactionRecord {
    get {return _transactionRecord ?? Proto_TransactionRecord()}
    set {_transactionRecord = newValue}
  }
  /// Returns true if `transactionRecord` has been explicitly set.
  public var hasTransactionRecord: Bool {return self._transactionRecord != nil}
  /// Clears the value of `transactionRecord`. Subsequent reads from it will return its default value.
  public mutating func clearTransactionRecord() {self._transactionRecord = nil}

  ///*
  /// The records of processing all consensus transaction with the same id as the distinguished
  /// record above, in chronological order.
  public var duplicateTransactionRecords: [Proto_TransactionRecord] = []

  ///*
  /// The records of processing all child transaction spawned by the transaction with the given
  /// top-level id, in consensus order. Always empty if the top-level status is UNKNOWN.
  public var childTransactionRecords: [Proto_TransactionRecord] = []

  public var unknownFields = SwiftProtobuf.UnknownStorage()

  public init() {}

  fileprivate var _header: Proto_ResponseHeader? = nil
  fileprivate var _transactionRecord: Proto_TransactionRecord? = nil
}

#if swift(>=5.5) && canImport(_Concurrency)
extension Proto_TransactionGetRecordQuery: @unchecked Sendable {}
extension Proto_TransactionGetRecordResponse: @unchecked Sendable {}
#endif  // swift(>=5.5) && canImport(_Concurrency)

// MARK: - Code below here is support for the SwiftProtobuf runtime.

fileprivate let _protobuf_package = "proto"

extension Proto_TransactionGetRecordQuery: SwiftProtobuf.Message, SwiftProtobuf._MessageImplementationBase, SwiftProtobuf._ProtoNameProviding {
  public static let protoMessageName: String = _protobuf_package + ".TransactionGetRecordQuery"
  public static let _protobuf_nameMap: SwiftProtobuf._NameMap = [
    1: .same(proto: "header"),
    2: .same(proto: "transactionID"),
    3: .same(proto: "includeDuplicates"),
    4: .standard(proto: "include_child_records"),
  ]

  public mutating func decodeMessage<D: SwiftProtobuf.Decoder>(decoder: inout D) throws {
    while let fieldNumber = try decoder.nextFieldNumber() {
      // The use of inline closures is to circumvent an issue where the compiler
      // allocates stack space for every case branch when no optimizations are
      // enabled. https://github.com/apple/swift-protobuf/issues/1034
      switch fieldNumber {
      case 1: try { try decoder.decodeSingularMessageField(value: &self._header) }()
      case 2: try { try decoder.decodeSingularMessageField(value: &self._transactionID) }()
      case 3: try { try decoder.decodeSingularBoolField(value: &self.includeDuplicates) }()
      case 4: try { try decoder.decodeSingularBoolField(value: &self.includeChildRecords) }()
      default: break
      }
    }
  }

  public func traverse<V: SwiftProtobuf.Visitor>(visitor: inout V) throws {
    // The use of inline closures is to circumvent an issue where the compiler
    // allocates stack space for every if/case branch local when no optimizations
    // are enabled. https://github.com/apple/swift-protobuf/issues/1034 and
    // https://github.com/apple/swift-protobuf/issues/1182
    try { if let v = self._header {
      try visitor.visitSingularMessageField(value: v, fieldNumber: 1)
    } }()
    try { if let v = self._transactionID {
      try visitor.visitSingularMessageField(value: v, fieldNumber: 2)
    } }()
    if self.includeDuplicates != false {
      try visitor.visitSingularBoolField(value: self.includeDuplicates, fieldNumber: 3)
    }
    if self.includeChildRecords != false {
      try visitor.visitSingularBoolField(value: self.includeChildRecords, fieldNumber: 4)
    }
    try unknownFields.traverse(visitor: &visitor)
  }

  public static func ==(lhs: Proto_TransactionGetRecordQuery, rhs: Proto_TransactionGetRecordQuery) -> Bool {
    if lhs._header != rhs._header {return false}
    if lhs._transactionID != rhs._transactionID {return false}
    if lhs.includeDuplicates != rhs.includeDuplicates {return false}
    if lhs.includeChildRecords != rhs.includeChildRecords {return false}
    if lhs.unknownFields != rhs.unknownFields {return false}
    return true
  }
}

extension Proto_TransactionGetRecordResponse: SwiftProtobuf.Message, SwiftProtobuf._MessageImplementationBase, SwiftProtobuf._ProtoNameProviding {
  public static let protoMessageName: String = _protobuf_package + ".TransactionGetRecordResponse"
  public static let _protobuf_nameMap: SwiftProtobuf._NameMap = [
    1: .same(proto: "header"),
    3: .same(proto: "transactionRecord"),
    4: .same(proto: "duplicateTransactionRecords"),
    5: .standard(proto: "child_transaction_records"),
  ]

  public mutating func decodeMessage<D: SwiftProtobuf.Decoder>(decoder: inout D) throws {
    while let fieldNumber = try decoder.nextFieldNumber() {
      // The use of inline closures is to circumvent an issue where the compiler
      // allocates stack space for every case branch when no optimizations are
      // enabled. https://github.com/apple/swift-protobuf/issues/1034
      switch fieldNumber {
      case 1: try { try decoder.decodeSingularMessageField(value: &self._header) }()
      case 3: try { try decoder.decodeSingularMessageField(value: &self._transactionRecord) }()
      case 4: try { try decoder.decodeRepeatedMessageField(value: &self.duplicateTransactionRecords) }()
      case 5: try { try decoder.decodeRepeatedMessageField(value: &self.childTransactionRecords) }()
      default: break
      }
    }
  }

  public func traverse<V: SwiftProtobuf.Visitor>(visitor: inout V) throws {
    // The use of inline closures is to circumvent an issue where the compiler
    // allocates stack space for every if/case branch local when no optimizations
    // are enabled. https://github.com/apple/swift-protobuf/issues/1034 and
    // https://github.com/apple/swift-protobuf/issues/1182
    try { if let v = self._header {
      try visitor.visitSingularMessageField(value: v, fieldNumber: 1)
    } }()
    try { if let v = self._transactionRecord {
      try visitor.visitSingularMessageField(value: v, fieldNumber: 3)
    } }()
    if !self.duplicateTransactionRecords.isEmpty {
      try visitor.visitRepeatedMessageField(value: self.duplicateTransactionRecords, fieldNumber: 4)
    }
    if !self.childTransactionRecords.isEmpty {
      try visitor.visitRepeatedMessageField(value: self.childTransactionRecords, fieldNumber: 5)
    }
    try unknownFields.traverse(visitor: &visitor)
  }

  public static func ==(lhs: Proto_TransactionGetRecordResponse, rhs: Proto_TransactionGetRecordResponse) -> Bool {
    if lhs._header != rhs._header {return false}
    if lhs._transactionRecord != rhs._transactionRecord {return false}
    if lhs.duplicateTransactionRecords != rhs.duplicateTransactionRecords {return false}
    if lhs.childTransactionRecords != rhs.childTransactionRecords {return false}
    if lhs.unknownFields != rhs.unknownFields {return false}
    return true
  }
}
