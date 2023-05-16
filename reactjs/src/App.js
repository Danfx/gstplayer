import './App.css';
import React from 'react'

function App() {
  return (
    <div className="App">
      <video controls autoPlay>
        <source src="http://localhost:8180" />
      </video>
    </div>
  );
}

export default App;
