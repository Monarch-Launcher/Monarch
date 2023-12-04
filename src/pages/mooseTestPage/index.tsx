import Button from '@_ui/button';
import Page from '@_ui/page';
import Spinner from '@_ui/spinner';
import { invoke } from '@tauri-apps/api';
import * as React from 'react';
import styled from 'styled-components';

const ResultContainer = styled.div`
  overflow-y: auto;
  height: calc(100% - 10rem);
`;

const Buttons = styled.div`
  display: flex;
  gap: 1rem;
`;

const MooseTestPage = () => {
  const [result, setResult] = React.useState<unknown>();
  const [loading, setLoading] = React.useState(false);
  const [logsError, setLogsError] = React.useState(false);

  // This function will be called when the button is clicked
  const handleClick = React.useCallback(async () => {
    try {
      setLoading(true);
      // -- invoke() parameters --
      // first parameters: string -> the name of the rust command to call (required)
      // second parameters: json object -> the parameter(s) the rust command takes (optional)
      // E.g. To call the function_name command: invoke('function_name', {arg1: 'foo', arg2: 'bar'})
      
      const functionResult = await invoke('download_game', {name: "CS2", platform: "steam", platformId: "730"});
      
      //const functionResult = await invoke('set_setting', {
      //  header: "test", key: "key", value: "value"
      //}) ;
      //
      //
      // Don't edit the rest of this code
      //
      // s
      setResult(functionResult);
    } catch (err) {
      setResult(err);
    } finally {
      setLoading(false);
    }
  }, []);

  const printLogs = React.useCallback(async () => {
    try {
      setLogsError(false);
      await invoke('open_logs');
    } catch (err) {
      setLogsError(true);
    }
  }, []);

  return (
    <Page title="Moose's test page">
      <Buttons>
        <Button
          variant="primary"
          type="button"
          onClick={handleClick}
          loading={loading}
        >
          Call command
        </Button>
        <Button
          variant="primary"
          type="button"
          onClick={printLogs}
          loading={loading}
        >
          Print logs
        </Button>
      </Buttons>

      <h3>Result of called rust command:</h3>
      <ResultContainer>
        {loading ? <Spinner /> : JSON.stringify(result)}
        {logsError && <p>There was an error opening logs</p>}
      </ResultContainer>
    </Page>
  );
};

export default MooseTestPage;
