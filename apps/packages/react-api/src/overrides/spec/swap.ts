// Copyright 2017-2019 @polkadot/react-api authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.
import { u128, u64 } from '@polkadot/types';

const types =  {
  FeeRate: typeof u64,
  AssetId: typeof u64,
  Balance: typeof u128,
};

export default {
  ...types,
};
