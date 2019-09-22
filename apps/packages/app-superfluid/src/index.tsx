// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

// some types, AppProps for the app and I18nProps to indicate
// translatable strings. Generally the latter is quite "light",
// `t` is inject into props (see the HOC export) and `t('any text')
// does the translation
import {AppProps, I18nProps} from '@polkadot/react-components/types';
// external imports (including those found in the packages/*
// of this repo)
import React, {useState} from 'react';
// local imports and components
import AccountSelector from "@polkadot/app-superfluid/componments/AccountSelector";
import translate from './translate';
import Container from "@polkadot/app-superfluid/componments/Container";

import './index.css'
import logo from './images/logo.jpg'

// define our internal types
interface Props extends AppProps, I18nProps {
}

function App({className, basePath}: Props): React.ReactElement<Props> {
  const [accountId, setAccountId] = useState<string | undefined>();

  return (
    // in all apps, the main wrapper is setup to allow the padding
    // and margins inside the application. (Just from a consistent pov)
    <main className={`${className} uniswap-content`}>
      <div style={{
        marginTop: '50px',
        textAlign: 'center'
      }}>
        <img style={{
          width: '250px',
          margin: '0 auto',
          display: 'inline-block',
          backgroundPosition: 'bottom',
          backgroundAttachment: 'fixed',
        }} src={logo} alt="SuperFluid"/>
      </div>
      <Container accountId={accountId} basePath={basePath}/>

      <div style={{display: 'none'}}>
        <AccountSelector onChange={setAccountId}/>
      </div>
    </main>
  );
}

export default translate(App);
