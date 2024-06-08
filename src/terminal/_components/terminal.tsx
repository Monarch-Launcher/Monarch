import { useEffect, useState, FormEvent } from 'react';
import { listen } from '@tauri-apps/api/event';

const Terminal = () => {
  const [output, setOutput] = useState<string[]>([]);
  const [input, setInput] = useState<string>('');

  useEffect(() => {
    // Listen for the command output event
    const unlisten = listen('stdout', event => {
      setOutput(prevOutput => [...prevOutput, event.payload as string]);
    });

    // Clean up the event listener on component unmount
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    // Handle the input submission (e.g., send it to the backend)
    // For now, we will just add it to the output
    setOutput(prevOutput => [...prevOutput, input]);
    setInput(''); // Clear the input field
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <div id="output-container" style={{ flexGrow: 1, overflowY: 'auto' }}>
        {output.map((line, index) => (
          <pre key={index} style={{ whiteSpace: 'pre-wrap' }}>{line}</pre>
        ))}
      </div>
      <form onSubmit={handleSubmit} style={{ marginTop: '10px' }}>
        <input 
          type="text" 
          value={input} 
          onChange={(e) => setInput(e.target.value)} 
          style={{ width: '100%', padding: '8px' }} 
          placeholder="Type your command here..."
        />
      </form>
    </div>
  );
}

export default Terminal;
