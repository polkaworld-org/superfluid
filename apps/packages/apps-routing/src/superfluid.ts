// Copyright 2017-2019 @polkadot/apps-routing authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import {Routes} from './types';

import Superfluid from '@polkadot/app-superfluid';

export default ([
  {
    Component: Superfluid,
    display: {
      isHidden: false,
      needsAccounts: true,
      needsApi: [
        'tx.balances.transfer'
      ]
    },
    i18n: {
      defaultValue: 'Superfluid'
    },
    icon: 'exchange',
    name: 'superfluid'
  }
] as Routes);
