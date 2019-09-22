// Copyright 2017-2019 @polkadot/app-storage authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import {ApiProps} from '@polkadot/react-api/types';
import {I18nProps} from '@polkadot/react-components/types';
import {TabItem} from '@polkadot/react-components/Tabs';

import React from 'react';
import {Route, Switch} from 'react-router';
import {Tabs} from '@polkadot/react-components';
import {withApi} from '@polkadot/react-api';

import Swap from './Swap';
import Pool from './Pool';
import translate from '../../translate';

interface Props extends ApiProps, I18nProps {
  basePath: string;
  accountId?: string;
}

interface State {
  items: TabItem[];
}

class Selection extends React.PureComponent<Props, State> {
  public constructor(props: Props) {
    super(props);

    const {t} = this.props;

    this.state = {
      items: [
        {
          isRoot: true,
          name: 'swap',
          text: t('Swap')
        },
        {
          name: 'pool',
          text: t('Pool')
        }
      ]
    };
  }

  async componentDidMount() {
    const {api} = this.props
    const assetNum = await api.query.superfluid.nextAssetId()
    console.log(assetNum.toString(), 'assetNum ------------')
  }

  public render(): React.ReactNode {
    const {basePath, accountId} = this.props;
    const {items} = this.state;
    console.log(basePath, 'uniswap  basePath')

    return (
      <>
        <header>
          <Tabs
            basePath={basePath}
            items={items}
          />
        </header>
        <Switch>
          <Route path={`${basePath}/pool`} render={() => (<Pool accountId={accountId}/>)}/>
          <Route render={() => (<Swap accountId={accountId}/>)}/>
        </Switch>
      </>
    );
  }
}

export default translate(withApi(Selection));
