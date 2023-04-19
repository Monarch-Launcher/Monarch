import * as React from 'react';
import styled from 'styled-components';
import { invoke } from '@tauri-apps/api';
import Page from '../../common/page';
import Button from '../../common/button';

const ResultContainer = styled.div`
  overflow-y: auto;
  height: calc(100% - 10rem);
`;

const MooseTestPage = () => {
  const [result, setResult] = React.useState<unknown>();

  // This function will be called when the button is clicked
  const handleClick = React.useCallback(async () => {
    try {
      // -- invoke() parameters --
      // first parameters: string -> the name of the rust command to call (required)
      // second parameters: json object -> the parameter(s) the rust command takes (optional)
      // E.g. To call the function_name command: invoke('function_name', {arg1: 'foo', arg2: 'bar'})
      const functionResult = await invoke('search_games', {
        name: 5,
      });
      //
      //
      // Don't edit the rest of this code
      //
      //
      setResult(functionResult);
    } catch (err) {
      setResult(err);
    }
  }, []);

  return (
    <Page title="Moose's test page">
      <Button variant="primary" type="button" onClick={handleClick}>
        Call command
      </Button>
      <h3>Result of called rust command:</h3>
      <ResultContainer>{JSON.stringify(result)}</ResultContainer>
    </Page>
  );
};

export default MooseTestPage;
