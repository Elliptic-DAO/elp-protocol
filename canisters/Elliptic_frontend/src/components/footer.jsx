import React from "react";

export default function Footer() {
    return (
        <footer>
            <nav style={{ height: 'fit-content', alignItems: 'center', paddingLeft: '2vw', paddingRight: '2vw' }}>
                <img src="/mobius_strip.png" style={{ height: 100 }} />
                <h1>Elliptic DAO</h1>
                <div>
                    <div style={{ display: 'flex', flexDirection: 'column', margin: '2.5em' }}>
                        <h1>Socials</h1>
                        <ul style={{ display: 'flex', flexDirection: 'column' }}>
                            <li><a href="https://twitter.com/elliptic_dao" target="_blank">Twitter</a></li>
                            <li><a href="https://oc.app/5gnc5-giaaa-aaaar-alooa-cai/?code=ef8c8f73321e5721" target="_blank">Open Chat</a></li>
                            {/* <li>DSCVR</li> */}
                        </ul>
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', margin: '2.5em' }}>
                        <h1>Ressources</h1>
                        <ul style={{ display: 'flex', flexDirection: 'column' }}>
                            <li><a href="https://github.com/Elliptic-DAO" target="_blank">Github</a></li>
                            <li><a href="/elliptic_dao_whitepaper.pdf" target="_blank">White Paper</a></li>
                        </ul>
                    </div>
                </div>
            </nav>
        </footer>

    );
}