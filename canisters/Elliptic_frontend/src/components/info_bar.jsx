import { use_local_state } from "../utils/state";

function numberToPercentage(number) {
    const percentage = (number * 100).toFixed(1);
    return `${percentage}%`;
}

function numberToDollars(number) {
    const percentage = (number).toFixed(1);
    return `${percentage}`;
}

export default function InfoBar() {
    let state = use_local_state();

    return (
        <nav style={{ height: "43px", paddingLeft: '10vw', alignItems: 'center', borderBottom: 'solid' }}>
            <b>ICP Price: ${state.icp_price.toFixed(2)}</b>
            <b style={{ marginLeft: '1em' }}>Total Value Locked: ${numberToDollars(state.tvl)}</b>
            <b style={{ marginLeft: '1em' }}>Collateral Ratio: {numberToPercentage(state.collateral_ratio)}</b>
            <div style={{ marginLeft: 'auto', marginRight: '10vw' }}>
                <a href="https://twitter.com/elliptic_dao" target="_blank">Twitter</a>
                <a href="https://github.com/Elliptic-DAO" target="_blank" style={{ paddingLeft: '1em' }}>Github</a>
            </div>
        </nav>
    );
}