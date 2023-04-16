import React from 'react';
import Navbar from './components/navbar';
import EllipticStatus from './elliptic_status';
import Footer from './components/footer';
import Actions from './actions';
import InfoBar from './components/info_bar';

function App() {
  return (
    <div>
      <InfoBar />
      <Navbar />
      <Actions />
      <Footer />
    </div>
  );
}

export default App;