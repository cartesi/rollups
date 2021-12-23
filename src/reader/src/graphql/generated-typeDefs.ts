import { GraphQLResolveInfo } from 'graphql';
export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type RequireFields<T, K extends keyof T> = { [X in Exclude<keyof T, K>]?: T[X] } & { [P in K]-?: NonNullable<T[P]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
};

export type AccumulatingEpoch = {
  __typename?: 'AccumulatingEpoch';
  descartesv2_contract_address: Scalars['String'];
  epoch_number: Scalars['String'];
  id: Scalars['ID'];
  input_contract_address: Scalars['String'];
  inputs: EpochInputState;
};

export type AccumulatingEpochInput = {
  descartesv2_contract_address: Scalars['String'];
  epoch_number: Scalars['String'];
  input_contract_address: Scalars['String'];
  inputs: EpochInputStateInput;
};

export type CartesiMachineHash = {
  __typename?: 'CartesiMachineHash';
  data: Scalars['String'];
};

export enum CompletionStatus {
  Accepted = 'ACCEPTED',
  CycleLimitExceeded = 'CYCLE_LIMIT_EXCEEDED',
  MachineHalted = 'MACHINE_HALTED',
  RejectedByMachine = 'REJECTED_BY_MACHINE',
  TimeLimitExceeded = 'TIME_LIMIT_EXCEEDED'
}

export type EpochInputState = {
  __typename?: 'EpochInputState';
  epoch_number: Scalars['String'];
  id: Scalars['ID'];
  input_contract_address: Scalars['String'];
  inputs: Array<Maybe<Input>>;
};

export type EpochInputStateInput = {
  epoch_number: Scalars['String'];
  input_contract_address: Scalars['String'];
  inputs: Array<Maybe<InputData>>;
};

export enum EpochState {
  Active = 'ACTIVE',
  Finished = 'FINISHED'
}

export type FinalizedEpoch = {
  __typename?: 'FinalizedEpoch';
  epoch_number: Scalars['String'];
  finalized_block_hash: Scalars['String'];
  finalized_block_number: Scalars['String'];
  hash: Scalars['String'];
  id: Scalars['ID'];
  inputs: EpochInputState;
};

export type FinalizedEpochInput = {
  epoch_number: Scalars['String'];
  finalized_block_hash: Scalars['String'];
  finalized_block_number: Scalars['String'];
  hash: Scalars['String'];
  inputs: EpochInputStateInput;
};

export type FinalizedEpochs = {
  __typename?: 'FinalizedEpochs';
  descartesv2_contract_address: Scalars['String'];
  finalized_epochs: Array<Maybe<FinalizedEpoch>>;
  id: Scalars['ID'];
  initial_epoch: Scalars['String'];
  input_contract_address: Scalars['String'];
};

export type FinalizedEpochsInput = {
  descartesv2_contract_address: Scalars['String'];
  finalized_epochs: Array<Maybe<FinalizedEpochInput>>;
  initial_epoch: Scalars['String'];
  input_contract_address: Scalars['String'];
};

export type GetEpochStatusRequest = {
  epoch_index: Scalars['String'];
  session_id: Scalars['String'];
};

export type GetEpochStatusResponse = {
  __typename?: 'GetEpochStatusResponse';
  epoch_index: Scalars['String'];
  most_recent_machine_hash: Scalars['String'];
  most_recent_notices_epoch_root_hash: Scalars['String'];
  most_recent_vouchers_epoch_root_hash: Scalars['String'];
  pending_input_count: Scalars['String'];
  processed_inputs: Array<Maybe<ProcessedInput>>;
  session_id: Scalars['String'];
  state: EpochState;
  taint_status: TaintStatus;
};

export type GetProcessedInputsVouchersAndNotices = {
  __typename?: 'GetProcessedInputsVouchersAndNotices';
  processed_input?: Maybe<ProcessedInput>;
  report?: Maybe<Report>;
  voucher?: Maybe<Voucher>;
};

export type GetSessionStatusRequest = {
  session_id: Scalars['ID'];
};

export type GetSessionStatusResponse = {
  __typename?: 'GetSessionStatusResponse';
  active_epoch_index: Scalars['Int'];
  epoch_index: Array<Maybe<Scalars['Int']>>;
  session_id: Scalars['ID'];
  taint_status: TaintStatus;
};

export type GetStatusResponse = {
  __typename?: 'GetStatusResponse';
  session_id: Array<Maybe<Scalars['String']>>;
};

export type Hash = {
  __typename?: 'Hash';
  data: Scalars['String'];
};

export type ImmutableState = {
  __typename?: 'ImmutableState';
  challenge_period: Scalars['String'];
  contract_creation_timestamp: Scalars['String'];
  descartesv2_contract_address: Scalars['String'];
  dispute_contract_address: Scalars['String'];
  id: Scalars['ID'];
  input_contract_address: Scalars['String'];
  input_duration: Scalars['String'];
  validator_contract_address: Scalars['String'];
  voucher_contract_address: Scalars['String'];
};

export type ImmutableStateInput = {
  challenge_period: Scalars['String'];
  descartesv2_contract_address: Scalars['String'];
  dispute_contract_address: Scalars['String'];
  input_contract_address: Scalars['String'];
  input_duration: Scalars['String'];
  validator_contract_address: Scalars['String'];
  voucher_contract_address: Scalars['String'];
};

export type Input = {
  __typename?: 'Input';
  id: Scalars['ID'];
  payload: Array<Maybe<Scalars['String']>>;
  sender: Scalars['String'];
  timestamp: Scalars['String'];
};

export type InputData = {
  payload: Array<Maybe<Scalars['String']>>;
  sender: Scalars['String'];
  timestamp: Scalars['String'];
};

export type InputResult = {
  __typename?: 'InputResult';
  epoch_index: Scalars['String'];
  input_index: Scalars['String'];
  notice_hashes_in_machine: MerkleTreeProof;
  notices: Array<Maybe<Notice>>;
  session_id: Scalars['String'];
  voucher_hashes_in_machine: MerkleTreeProof;
  vouchers: Array<Maybe<Voucher>>;
};

export type IntegerBool = {
  __typename?: 'IntegerBool';
  integer: Scalars['Boolean'];
};

export type IntegerBoolInput = {
  integer: Scalars['Boolean'];
};

export type IntegerInnerObject = {
  __typename?: 'IntegerInnerObject';
  integer?: Maybe<IntegerBool>;
};

export type IntegerInnerObjectInput = {
  integer: IntegerBoolInput;
};

export type IntegerObject = {
  __typename?: 'IntegerObject';
  integer?: Maybe<IntegerInnerObject>;
};

export type IntegerObjectInput = {
  integer: IntegerInnerObjectInput;
};

export type Keys = {
  epoch_index?: Maybe<Scalars['String']>;
  input_index?: Maybe<Scalars['String']>;
  session_id?: Maybe<Scalars['String']>;
};

export type MerkleTreeProof = {
  __typename?: 'MerkleTreeProof';
  id: Scalars['ID'];
  log2_root_size: Scalars['String'];
  log2_target_size: Scalars['String'];
  root_hash: Scalars['String'];
  sibling_hashes: Array<Maybe<Hash>>;
  target_address: Scalars['String'];
  target_hash: Scalars['String'];
};

export type Mutation = {
  __typename?: 'Mutation';
  RollupsState: RollupsState;
  constants: Array<Maybe<ImmutableState>>;
  current_epoch: AccumulatingEpoch;
  current_phase: PhaseState;
  finalized_epochs: Array<Maybe<FinalizedEpochs>>;
  initial_epoch: Scalars['String'];
  voucher_state: VoucherState;
};


export type MutationRollupsStateArgs = {
  input: RollupsInput;
};


export type MutationConstantsArgs = {
  input: Array<Maybe<ImmutableStateInput>>;
};


export type MutationCurrent_EpochArgs = {
  input: AccumulatingEpochInput;
};


export type MutationCurrent_PhaseArgs = {
  input: PhaseState;
};


export type MutationFinalized_EpochsArgs = {
  input: Array<Maybe<FinalizedEpochsInput>>;
};


export type MutationInitial_EpochArgs = {
  input: Scalars['String'];
};


export type MutationVoucher_StateArgs = {
  input: VoucherStateInput;
};

export type Notice = {
  __typename?: 'Notice';
  epoch_index: Scalars['String'];
  input_index: Scalars['String'];
  keccak: Scalars['String'];
  keccak_in_notice_hashes: MerkleTreeProof;
  notice_index: Scalars['String'];
  payload: Scalars['String'];
  session_id: Scalars['String'];
};

export type NoticeKeys = {
  epoch_index?: Maybe<Scalars['String']>;
  input_index?: Maybe<Scalars['String']>;
  notice_index?: Maybe<Scalars['String']>;
  session_id?: Maybe<Scalars['String']>;
};

export enum PhaseState {
  AwaitingConsensusAfterConflict = 'AwaitingConsensusAfterConflict',
  AwaitingConsensusNoConflict = 'AwaitingConsensusNoConflict',
  AwaitingDispute = 'AwaitingDispute',
  ConsensusTimeout = 'ConsensusTimeout',
  EpochSealedAwaitingFirstClaim = 'EpochSealedAwaitingFirstClaim',
  InputAccumulation = 'InputAccumulation'
}

export type ProcessedInput = {
  __typename?: 'ProcessedInput';
  epoch_index: Scalars['String'];
  input_index: Scalars['String'];
  most_recent_machine_hash: Scalars['String'];
  notice_hashes_in_epoch: MerkleTreeProof;
  reports: Array<Maybe<Report>>;
  result?: Maybe<InputResult>;
  session_id: Scalars['String'];
  skip_reason?: Maybe<CompletionStatus>;
  voucher_hashes_in_epoch: MerkleTreeProof;
};

export type Query = {
  __typename?: 'Query';
  GetEpochStatus: GetEpochStatusResponse;
  GetNotice?: Maybe<Array<Maybe<Notice>>>;
  GetProcessedInput?: Maybe<Array<Maybe<ProcessedInput>>>;
  GetSessionStatus: GetSessionStatusResponse;
  GetStatus: GetStatusResponse;
  GetVersion: Version;
  GetVoucher?: Maybe<Array<Maybe<Voucher>>>;
  RollupsState: Array<Maybe<RollupsState>>;
  constants: Array<Maybe<ImmutableState>>;
  current_epoch: Array<Maybe<AccumulatingEpoch>>;
  current_phase: Array<Maybe<PhaseState>>;
  finalized_epochs: Array<Maybe<FinalizedEpochs>>;
  initial_epoch: Scalars['String'];
  voucher_state: Array<Maybe<VoucherState>>;
};


export type QueryGetEpochStatusArgs = {
  query: GetEpochStatusRequest;
};


export type QueryGetNoticeArgs = {
  query?: Maybe<NoticeKeys>;
};


export type QueryGetProcessedInputArgs = {
  query?: Maybe<Keys>;
};


export type QueryGetSessionStatusArgs = {
  query: GetSessionStatusRequest;
};


export type QueryGetVoucherArgs = {
  query?: Maybe<VoucherKeys>;
};

export type Report = {
  __typename?: 'Report';
  payload: Scalars['String'];
};

export type RollupsInput = {
  block_hash: Scalars['String'];
  constants: ImmutableStateInput;
  current_epoch: AccumulatingEpochInput;
  current_phase: PhaseState;
  initial_epoch: Scalars['String'];
  voucher_state: VoucherStateInput;
};

export type RollupsState = {
  __typename?: 'RollupsState';
  block_hash: Scalars['String'];
  constants: ImmutableState;
  current_epoch: AccumulatingEpoch;
  current_phase: PhaseState;
  id: Scalars['ID'];
  initial_epoch: Scalars['String'];
  voucher_state: VoucherState;
};

export type TaintStatus = {
  __typename?: 'TaintStatus';
  error_code: Scalars['Int'];
  error_message: Scalars['String'];
};

export type Version = {
  __typename?: 'Version';
  id: Scalars['Int'];
  version: Scalars['String'];
};

export type Voucher = {
  __typename?: 'Voucher';
  Address: Scalars['String'];
  epoch_index: Scalars['String'];
  input_index: Scalars['String'];
  keccak: Scalars['String'];
  keccak_in_voucher_hashes: MerkleTreeProof;
  payload: Scalars['String'];
  session_id: Scalars['String'];
  voucher_index: Scalars['String'];
};

export type VoucherKeys = {
  epoch_index?: Maybe<Scalars['String']>;
  input_index?: Maybe<Scalars['String']>;
  session_id?: Maybe<Scalars['String']>;
  voucher_index?: Maybe<Scalars['String']>;
};

export type VoucherState = {
  __typename?: 'VoucherState';
  id: Scalars['ID'];
  voucher_address: Scalars['String'];
  vouchers?: Maybe<IntegerObject>;
};

export type VoucherStateInput = {
  voucher_address: Scalars['String'];
  vouchers: IntegerObjectInput;
};



export type ResolverTypeWrapper<T> = Promise<T> | T;


export type ResolverWithResolve<TResult, TParent, TContext, TArgs> = {
  resolve: ResolverFn<TResult, TParent, TContext, TArgs>;
};
export type Resolver<TResult, TParent = {}, TContext = {}, TArgs = {}> = ResolverFn<TResult, TParent, TContext, TArgs> | ResolverWithResolve<TResult, TParent, TContext, TArgs>;

export type ResolverFn<TResult, TParent, TContext, TArgs> = (
  parent: TParent,
  args: TArgs,
  context: TContext,
  info: GraphQLResolveInfo
) => Promise<TResult> | TResult;

export type SubscriptionSubscribeFn<TResult, TParent, TContext, TArgs> = (
  parent: TParent,
  args: TArgs,
  context: TContext,
  info: GraphQLResolveInfo
) => AsyncIterator<TResult> | Promise<AsyncIterator<TResult>>;

export type SubscriptionResolveFn<TResult, TParent, TContext, TArgs> = (
  parent: TParent,
  args: TArgs,
  context: TContext,
  info: GraphQLResolveInfo
) => TResult | Promise<TResult>;

export interface SubscriptionSubscriberObject<TResult, TKey extends string, TParent, TContext, TArgs> {
  subscribe: SubscriptionSubscribeFn<{ [key in TKey]: TResult }, TParent, TContext, TArgs>;
  resolve?: SubscriptionResolveFn<TResult, { [key in TKey]: TResult }, TContext, TArgs>;
}

export interface SubscriptionResolverObject<TResult, TParent, TContext, TArgs> {
  subscribe: SubscriptionSubscribeFn<any, TParent, TContext, TArgs>;
  resolve: SubscriptionResolveFn<TResult, any, TContext, TArgs>;
}

export type SubscriptionObject<TResult, TKey extends string, TParent, TContext, TArgs> =
  | SubscriptionSubscriberObject<TResult, TKey, TParent, TContext, TArgs>
  | SubscriptionResolverObject<TResult, TParent, TContext, TArgs>;

export type SubscriptionResolver<TResult, TKey extends string, TParent = {}, TContext = {}, TArgs = {}> =
  | ((...args: any[]) => SubscriptionObject<TResult, TKey, TParent, TContext, TArgs>)
  | SubscriptionObject<TResult, TKey, TParent, TContext, TArgs>;

export type TypeResolveFn<TTypes, TParent = {}, TContext = {}> = (
  parent: TParent,
  context: TContext,
  info: GraphQLResolveInfo
) => Maybe<TTypes> | Promise<Maybe<TTypes>>;

export type IsTypeOfResolverFn<T = {}, TContext = {}> = (obj: T, context: TContext, info: GraphQLResolveInfo) => boolean | Promise<boolean>;

export type NextResolverFn<T> = () => Promise<T>;

export type DirectiveResolverFn<TResult = {}, TParent = {}, TContext = {}, TArgs = {}> = (
  next: NextResolverFn<TResult>,
  parent: TParent,
  args: TArgs,
  context: TContext,
  info: GraphQLResolveInfo
) => TResult | Promise<TResult>;

/** Mapping between all available schema types and the resolvers types */
export type ResolversTypes = {
  AccumulatingEpoch: ResolverTypeWrapper<AccumulatingEpoch>;
  AccumulatingEpochInput: AccumulatingEpochInput;
  Boolean: ResolverTypeWrapper<Scalars['Boolean']>;
  CartesiMachineHash: ResolverTypeWrapper<CartesiMachineHash>;
  CompletionStatus: CompletionStatus;
  EpochInputState: ResolverTypeWrapper<EpochInputState>;
  EpochInputStateInput: EpochInputStateInput;
  EpochState: EpochState;
  FinalizedEpoch: ResolverTypeWrapper<FinalizedEpoch>;
  FinalizedEpochInput: FinalizedEpochInput;
  FinalizedEpochs: ResolverTypeWrapper<FinalizedEpochs>;
  FinalizedEpochsInput: FinalizedEpochsInput;
  GetEpochStatusRequest: GetEpochStatusRequest;
  GetEpochStatusResponse: ResolverTypeWrapper<GetEpochStatusResponse>;
  GetProcessedInputsVouchersAndNotices: ResolverTypeWrapper<GetProcessedInputsVouchersAndNotices>;
  GetSessionStatusRequest: GetSessionStatusRequest;
  GetSessionStatusResponse: ResolverTypeWrapper<GetSessionStatusResponse>;
  GetStatusResponse: ResolverTypeWrapper<GetStatusResponse>;
  Hash: ResolverTypeWrapper<Hash>;
  ID: ResolverTypeWrapper<Scalars['ID']>;
  ImmutableState: ResolverTypeWrapper<ImmutableState>;
  ImmutableStateInput: ImmutableStateInput;
  Input: ResolverTypeWrapper<Input>;
  InputData: InputData;
  InputResult: ResolverTypeWrapper<InputResult>;
  Int: ResolverTypeWrapper<Scalars['Int']>;
  IntegerBool: ResolverTypeWrapper<IntegerBool>;
  IntegerBoolInput: IntegerBoolInput;
  IntegerInnerObject: ResolverTypeWrapper<IntegerInnerObject>;
  IntegerInnerObjectInput: IntegerInnerObjectInput;
  IntegerObject: ResolverTypeWrapper<IntegerObject>;
  IntegerObjectInput: IntegerObjectInput;
  Keys: Keys;
  MerkleTreeProof: ResolverTypeWrapper<MerkleTreeProof>;
  Mutation: ResolverTypeWrapper<{}>;
  Notice: ResolverTypeWrapper<Notice>;
  NoticeKeys: NoticeKeys;
  PhaseState: PhaseState;
  ProcessedInput: ResolverTypeWrapper<ProcessedInput>;
  Query: ResolverTypeWrapper<{}>;
  Report: ResolverTypeWrapper<Report>;
  RollupsInput: RollupsInput;
  RollupsState: ResolverTypeWrapper<RollupsState>;
  String: ResolverTypeWrapper<Scalars['String']>;
  TaintStatus: ResolverTypeWrapper<TaintStatus>;
  Version: ResolverTypeWrapper<Version>;
  Voucher: ResolverTypeWrapper<Voucher>;
  VoucherKeys: VoucherKeys;
  VoucherState: ResolverTypeWrapper<VoucherState>;
  VoucherStateInput: VoucherStateInput;
};

/** Mapping between all available schema types and the resolvers parents */
export type ResolversParentTypes = {
  AccumulatingEpoch: AccumulatingEpoch;
  AccumulatingEpochInput: AccumulatingEpochInput;
  Boolean: Scalars['Boolean'];
  CartesiMachineHash: CartesiMachineHash;
  EpochInputState: EpochInputState;
  EpochInputStateInput: EpochInputStateInput;
  FinalizedEpoch: FinalizedEpoch;
  FinalizedEpochInput: FinalizedEpochInput;
  FinalizedEpochs: FinalizedEpochs;
  FinalizedEpochsInput: FinalizedEpochsInput;
  GetEpochStatusRequest: GetEpochStatusRequest;
  GetEpochStatusResponse: GetEpochStatusResponse;
  GetProcessedInputsVouchersAndNotices: GetProcessedInputsVouchersAndNotices;
  GetSessionStatusRequest: GetSessionStatusRequest;
  GetSessionStatusResponse: GetSessionStatusResponse;
  GetStatusResponse: GetStatusResponse;
  Hash: Hash;
  ID: Scalars['ID'];
  ImmutableState: ImmutableState;
  ImmutableStateInput: ImmutableStateInput;
  Input: Input;
  InputData: InputData;
  InputResult: InputResult;
  Int: Scalars['Int'];
  IntegerBool: IntegerBool;
  IntegerBoolInput: IntegerBoolInput;
  IntegerInnerObject: IntegerInnerObject;
  IntegerInnerObjectInput: IntegerInnerObjectInput;
  IntegerObject: IntegerObject;
  IntegerObjectInput: IntegerObjectInput;
  Keys: Keys;
  MerkleTreeProof: MerkleTreeProof;
  Mutation: {};
  Notice: Notice;
  NoticeKeys: NoticeKeys;
  ProcessedInput: ProcessedInput;
  Query: {};
  Report: Report;
  RollupsInput: RollupsInput;
  RollupsState: RollupsState;
  String: Scalars['String'];
  TaintStatus: TaintStatus;
  Version: Version;
  Voucher: Voucher;
  VoucherKeys: VoucherKeys;
  VoucherState: VoucherState;
  VoucherStateInput: VoucherStateInput;
};

export type AccumulatingEpochResolvers<ContextType = any, ParentType extends ResolversParentTypes['AccumulatingEpoch'] = ResolversParentTypes['AccumulatingEpoch']> = {
  descartesv2_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  epoch_number?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  input_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  inputs?: Resolver<ResolversTypes['EpochInputState'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type CartesiMachineHashResolvers<ContextType = any, ParentType extends ResolversParentTypes['CartesiMachineHash'] = ResolversParentTypes['CartesiMachineHash']> = {
  data?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type EpochInputStateResolvers<ContextType = any, ParentType extends ResolversParentTypes['EpochInputState'] = ResolversParentTypes['EpochInputState']> = {
  epoch_number?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  input_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  inputs?: Resolver<Array<Maybe<ResolversTypes['Input']>>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type FinalizedEpochResolvers<ContextType = any, ParentType extends ResolversParentTypes['FinalizedEpoch'] = ResolversParentTypes['FinalizedEpoch']> = {
  epoch_number?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  finalized_block_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  finalized_block_number?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  inputs?: Resolver<ResolversTypes['EpochInputState'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type FinalizedEpochsResolvers<ContextType = any, ParentType extends ResolversParentTypes['FinalizedEpochs'] = ResolversParentTypes['FinalizedEpochs']> = {
  descartesv2_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  finalized_epochs?: Resolver<Array<Maybe<ResolversTypes['FinalizedEpoch']>>, ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  initial_epoch?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type GetEpochStatusResponseResolvers<ContextType = any, ParentType extends ResolversParentTypes['GetEpochStatusResponse'] = ResolversParentTypes['GetEpochStatusResponse']> = {
  epoch_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  most_recent_machine_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  most_recent_notices_epoch_root_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  most_recent_vouchers_epoch_root_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  pending_input_count?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  processed_inputs?: Resolver<Array<Maybe<ResolversTypes['ProcessedInput']>>, ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  state?: Resolver<ResolversTypes['EpochState'], ParentType, ContextType>;
  taint_status?: Resolver<ResolversTypes['TaintStatus'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type GetProcessedInputsVouchersAndNoticesResolvers<ContextType = any, ParentType extends ResolversParentTypes['GetProcessedInputsVouchersAndNotices'] = ResolversParentTypes['GetProcessedInputsVouchersAndNotices']> = {
  processed_input?: Resolver<Maybe<ResolversTypes['ProcessedInput']>, ParentType, ContextType>;
  report?: Resolver<Maybe<ResolversTypes['Report']>, ParentType, ContextType>;
  voucher?: Resolver<Maybe<ResolversTypes['Voucher']>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type GetSessionStatusResponseResolvers<ContextType = any, ParentType extends ResolversParentTypes['GetSessionStatusResponse'] = ResolversParentTypes['GetSessionStatusResponse']> = {
  active_epoch_index?: Resolver<ResolversTypes['Int'], ParentType, ContextType>;
  epoch_index?: Resolver<Array<Maybe<ResolversTypes['Int']>>, ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  taint_status?: Resolver<ResolversTypes['TaintStatus'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type GetStatusResponseResolvers<ContextType = any, ParentType extends ResolversParentTypes['GetStatusResponse'] = ResolversParentTypes['GetStatusResponse']> = {
  session_id?: Resolver<Array<Maybe<ResolversTypes['String']>>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type HashResolvers<ContextType = any, ParentType extends ResolversParentTypes['Hash'] = ResolversParentTypes['Hash']> = {
  data?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type ImmutableStateResolvers<ContextType = any, ParentType extends ResolversParentTypes['ImmutableState'] = ResolversParentTypes['ImmutableState']> = {
  challenge_period?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  contract_creation_timestamp?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  descartesv2_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  dispute_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  input_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_duration?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  validator_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  voucher_contract_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type InputResolvers<ContextType = any, ParentType extends ResolversParentTypes['Input'] = ResolversParentTypes['Input']> = {
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  payload?: Resolver<Array<Maybe<ResolversTypes['String']>>, ParentType, ContextType>;
  sender?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  timestamp?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type InputResultResolvers<ContextType = any, ParentType extends ResolversParentTypes['InputResult'] = ResolversParentTypes['InputResult']> = {
  epoch_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  notice_hashes_in_machine?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  notices?: Resolver<Array<Maybe<ResolversTypes['Notice']>>, ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  voucher_hashes_in_machine?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  vouchers?: Resolver<Array<Maybe<ResolversTypes['Voucher']>>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type IntegerBoolResolvers<ContextType = any, ParentType extends ResolversParentTypes['IntegerBool'] = ResolversParentTypes['IntegerBool']> = {
  integer?: Resolver<ResolversTypes['Boolean'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type IntegerInnerObjectResolvers<ContextType = any, ParentType extends ResolversParentTypes['IntegerInnerObject'] = ResolversParentTypes['IntegerInnerObject']> = {
  integer?: Resolver<Maybe<ResolversTypes['IntegerBool']>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type IntegerObjectResolvers<ContextType = any, ParentType extends ResolversParentTypes['IntegerObject'] = ResolversParentTypes['IntegerObject']> = {
  integer?: Resolver<Maybe<ResolversTypes['IntegerInnerObject']>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type MerkleTreeProofResolvers<ContextType = any, ParentType extends ResolversParentTypes['MerkleTreeProof'] = ResolversParentTypes['MerkleTreeProof']> = {
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  log2_root_size?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  log2_target_size?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  root_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  sibling_hashes?: Resolver<Array<Maybe<ResolversTypes['Hash']>>, ParentType, ContextType>;
  target_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  target_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type MutationResolvers<ContextType = any, ParentType extends ResolversParentTypes['Mutation'] = ResolversParentTypes['Mutation']> = {
  RollupsState?: Resolver<ResolversTypes['RollupsState'], ParentType, ContextType, RequireFields<MutationRollupsStateArgs, 'input'>>;
  constants?: Resolver<Array<Maybe<ResolversTypes['ImmutableState']>>, ParentType, ContextType, RequireFields<MutationConstantsArgs, 'input'>>;
  current_epoch?: Resolver<ResolversTypes['AccumulatingEpoch'], ParentType, ContextType, RequireFields<MutationCurrent_EpochArgs, 'input'>>;
  current_phase?: Resolver<ResolversTypes['PhaseState'], ParentType, ContextType, RequireFields<MutationCurrent_PhaseArgs, 'input'>>;
  finalized_epochs?: Resolver<Array<Maybe<ResolversTypes['FinalizedEpochs']>>, ParentType, ContextType, RequireFields<MutationFinalized_EpochsArgs, 'input'>>;
  initial_epoch?: Resolver<ResolversTypes['String'], ParentType, ContextType, RequireFields<MutationInitial_EpochArgs, 'input'>>;
  voucher_state?: Resolver<ResolversTypes['VoucherState'], ParentType, ContextType, RequireFields<MutationVoucher_StateArgs, 'input'>>;
};

export type NoticeResolvers<ContextType = any, ParentType extends ResolversParentTypes['Notice'] = ResolversParentTypes['Notice']> = {
  epoch_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  keccak?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  keccak_in_notice_hashes?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  notice_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  payload?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type ProcessedInputResolvers<ContextType = any, ParentType extends ResolversParentTypes['ProcessedInput'] = ResolversParentTypes['ProcessedInput']> = {
  epoch_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  most_recent_machine_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  notice_hashes_in_epoch?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  reports?: Resolver<Array<Maybe<ResolversTypes['Report']>>, ParentType, ContextType>;
  result?: Resolver<Maybe<ResolversTypes['InputResult']>, ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  skip_reason?: Resolver<Maybe<ResolversTypes['CompletionStatus']>, ParentType, ContextType>;
  voucher_hashes_in_epoch?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type QueryResolvers<ContextType = any, ParentType extends ResolversParentTypes['Query'] = ResolversParentTypes['Query']> = {
  GetEpochStatus?: Resolver<ResolversTypes['GetEpochStatusResponse'], ParentType, ContextType, RequireFields<QueryGetEpochStatusArgs, 'query'>>;
  GetNotice?: Resolver<Maybe<Array<Maybe<ResolversTypes['Notice']>>>, ParentType, ContextType, RequireFields<QueryGetNoticeArgs, never>>;
  GetProcessedInput?: Resolver<Maybe<Array<Maybe<ResolversTypes['ProcessedInput']>>>, ParentType, ContextType, RequireFields<QueryGetProcessedInputArgs, never>>;
  GetSessionStatus?: Resolver<ResolversTypes['GetSessionStatusResponse'], ParentType, ContextType, RequireFields<QueryGetSessionStatusArgs, 'query'>>;
  GetStatus?: Resolver<ResolversTypes['GetStatusResponse'], ParentType, ContextType>;
  GetVersion?: Resolver<ResolversTypes['Version'], ParentType, ContextType>;
  GetVoucher?: Resolver<Maybe<Array<Maybe<ResolversTypes['Voucher']>>>, ParentType, ContextType, RequireFields<QueryGetVoucherArgs, never>>;
  RollupsState?: Resolver<Array<Maybe<ResolversTypes['RollupsState']>>, ParentType, ContextType>;
  constants?: Resolver<Array<Maybe<ResolversTypes['ImmutableState']>>, ParentType, ContextType>;
  current_epoch?: Resolver<Array<Maybe<ResolversTypes['AccumulatingEpoch']>>, ParentType, ContextType>;
  current_phase?: Resolver<Array<Maybe<ResolversTypes['PhaseState']>>, ParentType, ContextType>;
  finalized_epochs?: Resolver<Array<Maybe<ResolversTypes['FinalizedEpochs']>>, ParentType, ContextType>;
  initial_epoch?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  voucher_state?: Resolver<Array<Maybe<ResolversTypes['VoucherState']>>, ParentType, ContextType>;
};

export type ReportResolvers<ContextType = any, ParentType extends ResolversParentTypes['Report'] = ResolversParentTypes['Report']> = {
  payload?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type RollupsStateResolvers<ContextType = any, ParentType extends ResolversParentTypes['RollupsState'] = ResolversParentTypes['RollupsState']> = {
  block_hash?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  constants?: Resolver<ResolversTypes['ImmutableState'], ParentType, ContextType>;
  current_epoch?: Resolver<ResolversTypes['AccumulatingEpoch'], ParentType, ContextType>;
  current_phase?: Resolver<ResolversTypes['PhaseState'], ParentType, ContextType>;
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  initial_epoch?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  voucher_state?: Resolver<ResolversTypes['VoucherState'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type TaintStatusResolvers<ContextType = any, ParentType extends ResolversParentTypes['TaintStatus'] = ResolversParentTypes['TaintStatus']> = {
  error_code?: Resolver<ResolversTypes['Int'], ParentType, ContextType>;
  error_message?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type VersionResolvers<ContextType = any, ParentType extends ResolversParentTypes['Version'] = ResolversParentTypes['Version']> = {
  id?: Resolver<ResolversTypes['Int'], ParentType, ContextType>;
  version?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type VoucherResolvers<ContextType = any, ParentType extends ResolversParentTypes['Voucher'] = ResolversParentTypes['Voucher']> = {
  Address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  epoch_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  input_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  keccak?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  keccak_in_voucher_hashes?: Resolver<ResolversTypes['MerkleTreeProof'], ParentType, ContextType>;
  payload?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  session_id?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  voucher_index?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type VoucherStateResolvers<ContextType = any, ParentType extends ResolversParentTypes['VoucherState'] = ResolversParentTypes['VoucherState']> = {
  id?: Resolver<ResolversTypes['ID'], ParentType, ContextType>;
  voucher_address?: Resolver<ResolversTypes['String'], ParentType, ContextType>;
  vouchers?: Resolver<Maybe<ResolversTypes['IntegerObject']>, ParentType, ContextType>;
  __isTypeOf?: IsTypeOfResolverFn<ParentType, ContextType>;
};

export type Resolvers<ContextType = any> = {
  AccumulatingEpoch?: AccumulatingEpochResolvers<ContextType>;
  CartesiMachineHash?: CartesiMachineHashResolvers<ContextType>;
  EpochInputState?: EpochInputStateResolvers<ContextType>;
  FinalizedEpoch?: FinalizedEpochResolvers<ContextType>;
  FinalizedEpochs?: FinalizedEpochsResolvers<ContextType>;
  GetEpochStatusResponse?: GetEpochStatusResponseResolvers<ContextType>;
  GetProcessedInputsVouchersAndNotices?: GetProcessedInputsVouchersAndNoticesResolvers<ContextType>;
  GetSessionStatusResponse?: GetSessionStatusResponseResolvers<ContextType>;
  GetStatusResponse?: GetStatusResponseResolvers<ContextType>;
  Hash?: HashResolvers<ContextType>;
  ImmutableState?: ImmutableStateResolvers<ContextType>;
  Input?: InputResolvers<ContextType>;
  InputResult?: InputResultResolvers<ContextType>;
  IntegerBool?: IntegerBoolResolvers<ContextType>;
  IntegerInnerObject?: IntegerInnerObjectResolvers<ContextType>;
  IntegerObject?: IntegerObjectResolvers<ContextType>;
  MerkleTreeProof?: MerkleTreeProofResolvers<ContextType>;
  Mutation?: MutationResolvers<ContextType>;
  Notice?: NoticeResolvers<ContextType>;
  ProcessedInput?: ProcessedInputResolvers<ContextType>;
  Query?: QueryResolvers<ContextType>;
  Report?: ReportResolvers<ContextType>;
  RollupsState?: RollupsStateResolvers<ContextType>;
  TaintStatus?: TaintStatusResolvers<ContextType>;
  Version?: VersionResolvers<ContextType>;
  Voucher?: VoucherResolvers<ContextType>;
  VoucherState?: VoucherStateResolvers<ContextType>;
};

