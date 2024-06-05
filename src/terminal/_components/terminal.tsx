import Page from '@_ui/miniwindowPage';
import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

const Terminal = () => {
  const [output, setOutput] = useState([]);

  useEffect(() => {
    // Listen for the command output event
    const unlisten = listen('stdout', event => {
      setOutput(prevOutput => [...prevOutput, event.payload]);
    });

    // Clean up the event listener on component unmount
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  return (
    <div>
      <div id="output-container">
        {output.map((line, index) => (
          <pre key={index}>{line}</pre>
        ))}
      </div>
    </div>
  );
}

export default Terminal;
