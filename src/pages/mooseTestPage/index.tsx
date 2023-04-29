import * as React from 'react';
import styled from 'styled-components';
import { invoke } from '@tauri-apps/api';
import Page from '@_ui/page';
import Button from '@_ui/button';
import Spinner from '@_ui/spinner';

const ResultContainer = styled.div`
  overflow-y: auto;
  height: calc(100% - 10rem);
`;

const MooseTestPage = () => {
  const [result, setResult] = React.useState<unknown>();
  const [loading, setLoading] = React.useState(false);

  // This function will be called when the button is clicked
  const handleClick = React.useCallback(async () => {
    try {
      setLoading(true);
      // -- invoke() parameters --
      // first parameters: string -> the name of the rust command to call (required)
      // second parameters: json object -> the parameter(s) the rust command takes (optional)
      // E.g. To call the function_name command: invoke('function_name', {arg1: 'foo', arg2: 'bar'})
      const functionResult = await invoke('search_games', {
        name: 'worldbox - god simulator',
      });
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

  return (
    <Page title="Moose's test page">
      <Button
        variant="primary"
        type="button"
        onClick={handleClick}
        loading={loading}
      >
        Call command
      </Button>
      <h3>Result of called rust command:</h3>
      <ResultContainer>
        {loading ? <Spinner /> : JSON.stringify(result)}
      </ResultContainer>
    </Page>
  );
};

export default MooseTestPage;
