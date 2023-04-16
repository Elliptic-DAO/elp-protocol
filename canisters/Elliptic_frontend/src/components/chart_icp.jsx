import { ResponsiveLine } from '@nivo/line';
import React, { useState, useEffect } from "react";
import { use_local_state } from '../utils/state';

function create_price_entries(n) {
    let vector = [];
    for (let i = 0; i < n; i++) {
        let entry = {
            "x": i,
            "y": 5 + Math.random()
        };
        vector.push(entry);
    }
    return vector;
}

export default function IcpChart() {
    let state = use_local_state();

    let data_to_plot = [
        {
            "id": "icp price",
            "color": "hsl(129, 70%, 50%)",
            "data": create_price_entries(10)
        }
    ];

    return (
        <div style={{ height: 300, width: '68vw', margin: 'auto' }}>
            <ResponsiveLine
                data={data_to_plot}
                margin={{ top: 50, right: 110, bottom: 50, left: 60 }}
                xScale={{ type: 'point' }}
                yScale={{
                    type: 'linear',
                    min: 'auto',
                    max: 'auto',
                    stacked: true,
                    reverse: false
                }}
                yFormat=" >-.2f"
                axisTop={null}
                axisRight={null}
                axisBottom={{
                    orient: 'bottom',
                    tickSize: 5,
                    tickPadding: 5,
                    tickRotation: 0,
                    legend: 'time',
                    legendOffset: 36,
                    legendPosition: 'middle'
                }}
                axisLeft={{
                    orient: 'left',
                    tickSize: 5,
                    tickPadding: 5,
                    tickRotation: 0,
                    legend: 'ICP price',
                    legendOffset: -40,
                    legendPosition: 'middle'
                }}
                pointSize={6}
                pointColor={{ theme: 'background' }}
                pointBorderWidth={2}
                pointBorderColor={{ from: 'serieColor' }}
                pointLabelYOffset={-12}
                useMesh={true}
            />
        </div>

    );
}
