import React, { useState } from 'react';
import Swap from './components/actions/swap';
import Leverage from './components/actions/leverage';
import Liquidity from './components/actions/liquidity';
import { use_local_state } from './utils/state';
import Login from "./components/login";
import Analytics from './components/statistics';

export default function Actions() {
  let state = use_local_state();

  return (
    <div className="container">
      {state.active_component === "swap" && <Swap />}
      {state.active_component === "leverage" && <Leverage />}
      {state.active_component === "liquidity" && <Liquidity />}
      {state.active_component === "statistics" && <Analytics />}
      {state.active_component === "auth" && <Login />}
    </div>
  );
}