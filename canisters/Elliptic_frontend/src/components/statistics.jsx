import EllipticStatus from '../elliptic_status';
import React, { useState, useEffect } from "react";
import { use_provide_auth } from "../utils/auth";
import { use_local_state } from '../utils/state';
import IcpChart from './chart_icp';

export default function Analytics() {
    let state = use_local_state();

    return (
        <div>
            <IcpChart />
            <EllipticStatus />
        </div>
    );
}
