import { useEffect, useState, FormEvent } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

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

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    // Send the input event to the backend
    try {
      await invoke('write_process_stdin', { stdin: input});
      // Add the input to the output for display
      setOutput(prevOutput => [...prevOutput, input]);
      setInput(''); // Clear the input field
    } catch (error) {
      console.log(error);
    }

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
