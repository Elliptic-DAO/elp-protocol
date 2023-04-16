use crate::tasks::get_task_vec;
use crate::{read_state, E8S_FLOAT};
use std::io::Write;

pub fn build_dashboard() -> Vec<u8> {
    format!(
        "
    <!DOCTYPE html>
    <html lang=\"en\">
        <head>
            <title>Elliptic Protocol Dashboard</title>
            <style>
                table {{
                    border: solid;
                    text-align: left;
                    width: 100%;
                    border-width: thin;
                }}
                h3 {{
                    font-variant: small-caps;
                    margin-top: 30px;
                    margin-bottom: 5px;
                }}
                table table {{ font-size: small; }}
                .background {{ margin: 0; padding: 0; }}
                .content {{ max-width: 100vw; width: fit-content; margin: 0 auto; }}
                tbody tr:nth-child(odd) {{ background-color: #eeeeee; }}
            </style>
        </head>
        <body>
            <div>
                <h3>Metadata</h3>
                {}
            </div>
            <div>
                <h3>Liquidity Table</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Owner</th>
                            <th>Amount</th>
                        </tr>
                    </thead>
                    <tbody>{}</tbody>
                </table>
            </div>
            <div>
                <h3>Liquidity Rewards</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Owner</th>
                            <th>Amount</th>
                        </tr>
                    </thead>
                    <tbody>{}</tbody>
                </table>
            </div>
            <div>
                <h3>Leverage Table</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Owner</th>
                            <th>Amount</th>
                            <th>Take Profit</th>
                            <th>Timestamp</th>
                            <th>Icp Price Entry</th>
                            <th>Covered Amount</th>
                        </tr>
                    </thead>
                    <tbody>{}</tbody>
                </table>
            </div>
            <div>
                <h3>Swap Table</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Timestamp</th>
                            <th>Owner</th>
                            <th>From Block Index</th>
                            <th>From</th>
                            <th>To</th>
                        </tr>
                    </thead>
                    <tbody>{}</tbody>
                </table>
            </div>
            <div>
                <h3>Tasks queue</h3>
                <ul>
                    {}
                </ul>
            </div>
            <div>
                <h3>Fee Table</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Action</th>
                            <th>Fee</th>
                        </tr>
                    </thead>
                    <tbody>{}</tbody>
                </table>
            </div>
            <h3>Logs</h3>
            <table>
                <thead>
                    <tr><th>Priority</th><th>Timestamp</th><th>Location</th><th>Message</th></tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </body>
    </html>
    ",
        construct_metadata_table(),
        construct_liquidity_table(),
        construct_liquidity_rewards(),
        construct_leverage_table(),
        construct_convert_table(),
        construct_task_queue(),
        construct_fee_table(),
        display_logs()
    )
    .into_bytes()
}

fn with_utf8_buffer(f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut buf = Vec::new();
    f(&mut buf);
    String::from_utf8(buf).unwrap()
}

fn construct_metadata_table() -> String {
    read_state(|s| {
        format!(
            "<table>
                <tbody>
                    <tr>
                        <th>Mode</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>eusd ledger principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>icp ledger principal</th>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <th>xrc principal</th>
                        <td>{}</td>
                    </tr>
                    
                </tbody>
            </table>",
            s.mode, s.eusd_ledger_principal, s.icp_ledger_principal, s.xrc_principal,
        )
    })
}

fn construct_liquidity_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (principal, amount) in s.liquidity_provided.iter() {
                write!(
                    buf,
                    "
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                ",
                    principal, amount
                )
                .unwrap();
            }
        });
    })
}

fn construct_task_queue() -> String {
    with_utf8_buffer(|buf| {
        for task in get_task_vec() {
            write!(buf, "<li>{:?}</li>", task).unwrap()
        }
    })
}

fn construct_leverage_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (owner, leverages) in s.leverage_positions.iter() {
                for (i, leverage_position) in leverages.iter().enumerate() {
                    write!(buf, "<tr>").unwrap();
                    if i == 0 {
                        write!(buf, "<td rowspan='{}'>{}</td>", leverages.len(), owner).unwrap();
                    }
                    write!(
                        buf,
                        "<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        leverage_position.amount as f64 / E8S_FLOAT,
                        leverage_position.take_profit as f64 / E8S_FLOAT,
                        leverage_position.timestamp,
                        leverage_position.icp_entry_price.rate as f64 / E8S_FLOAT,
                        leverage_position.covered_amount as f64 / E8S_FLOAT
                    )
                    .unwrap()
                }
            }
        })
    })
}

fn construct_liquidity_rewards() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (principal, amount) in s.liquidity_rewards.iter() {
                write!(
                    buf,
                    "<tr><td>{}</td><td>{}</td></tr>",
                    principal,
                    *amount as f64 / E8S_FLOAT
                )
                .unwrap();
            }
        })
    })
}

fn construct_convert_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            for (_, conversion) in s.open_swaps.iter() {
                write!(
                    buf,
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td><td>{:?}</td></tr>",
                    conversion.timestamp,
                    conversion.caller,
                    conversion.from_block_index,
                    conversion.from,
                    conversion.to
                )
                .unwrap();
            }
        })
    })
}

fn construct_fee_table() -> String {
    with_utf8_buffer(|buf| {
        read_state(|s| {
            write!(
                buf,
                "<tr><td>Base Fee</td><td>{}%</td></tr>
                <tr><td>Liquidation Fee</td><td>{}%</td></tr>
                <tr><td>Stability Fee</td><td>{}%</td></tr>
                ",
                s.fees.base_fee as f64 / 100_000_000.0,
                s.fees.liquidation_fee as f64 / 100_000_000.0,
                s.fees.stability_fee as f64 / 100_000_000.0
            )
            .unwrap();
        })
    })
}

fn display_logs() -> String {
    use crate::logs::{P0, P1};
    use ic_canister_log::{export, LogEntry};

    fn display_entry(buf: &mut Vec<u8>, tag: &str, e: &LogEntry) {
        write!(
            buf,
            "<tr><td>{}</td><td class=\"ts-class\">{}</td><td><code>{}:{}</code></td><td>{}</td></tr>",
            tag, e.timestamp, e.file, e.line, e.message
        )
        .unwrap()
    }

    let p0 = export(&P0);
    let p1 = export(&P1);

    let mut i0 = 0;
    let mut i1 = 0;

    with_utf8_buffer(|buf| {
        // Merge sorted log entries with different priorities.
        while i0 < p0.len() && i1 < p1.len() {
            if p0[i0].timestamp <= p1[i1].timestamp {
                display_entry(buf, "P0", &p0[i0]);
                i0 += 1;
            } else {
                display_entry(buf, "P1", &p1[i1]);
                i1 += 1;
            }
        }

        for e in p0[i0..].iter() {
            display_entry(buf, "P0", e);
        }
        for e in p1[i1..].iter() {
            display_entry(buf, "P1", e);
        }
    })
}
