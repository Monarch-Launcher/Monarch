import Button from '@_ui/button';
import Modal from '@_ui/modal';
import { MdClose } from '@global/icons';
import * as React from 'react';
import styled from 'styled-components';

const ModalHeaderContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
  color: #fff;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: #fff;
`;

const ModalButtons = styled.div`
  display: flex;
  justify-content: right;
  align-items: center;
  gap: 1rem;
  margin: 2rem 0 1rem;
  color: #fff;
`;

const ModalContentContainer = styled.div`
  color: #fff;
  margin-right: 1.75rem;
`;

const FormGroup = styled.div`
  margin-bottom: 1.5rem;
`;

const Label = styled.label`
  display: block;
  margin-bottom: 0.5rem;
  color: #fff;
  font-weight: 600;
`;

const Input = styled.input`
  width: 100%;
  padding: 0.75rem;
  border: 2px solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  color: #fff;
  font-size: 1rem;

  &:focus {
    outline: none;
    border-color: #fa5002;
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.secondary};
  }
`;

const TextArea = styled.textarea`
  width: 100%;
  padding: 0.75rem;
  border: 2px solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  color: #fff;
  font-size: 1rem;
  min-height: 80px;
  resize: vertical;
  font-family: inherit;

  &:focus {
    outline: none;
    border-color: #fa5002;
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.secondary};
  }
`;

const ErrorText = styled.p`
  margin: 0.2rem 0 0 0;
  color: ${({ theme }) => theme.colors.error};
`;

type Props = {
  opened: boolean;
  close: () => void;
  selectedFilePath?: string;
};

export default ({ opened, close, selectedFilePath }: Props) => {
  const [gameName, setGameName] = React.useState('');
  const [errorMessage, setErrorMessage] = React.useState<string | undefined>();

  const handleGameNameChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setGameName(e.target.value);
      setErrorMessage(undefined);
    },
    [],
  );

  const handleAddGame = React.useCallback(async () => {
    if (gameName.length === 0) {
      setErrorMessage('Game name must be at least 1 character.');
      return;
    }

    if (!selectedFilePath) {
      setErrorMessage('No file selected.');
      return;
    }

    // TODO: Invoke function that adds the game with metadata
    // await invoke('add_game_manually', {
    //   name: gameName,
    //   executablePath: selectedFilePath
    // });

    console.log('Adding game:', { name: gameName, path: selectedFilePath });

    close();
    setGameName('');
    setErrorMessage(undefined);
  }, [close, gameName, selectedFilePath]);

  const handleCancel = React.useCallback(() => {
    close();
    setGameName('');
    setErrorMessage(undefined);
  }, [close]);

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>Add Game Manually</ModalHeader>
      </ModalHeaderContainer>
    );
  }, []);

  return (
    <Modal
      title={modalHeader}
      opened={opened}
      onClose={handleCancel}
      centered
      withCloseButton={false}
      size="900px"
    >
      <ModalContentContainer>
        <FormGroup>
          <Label htmlFor="gameName">Game Name</Label>
          <Input
            id="gameName"
            type="text"
            value={gameName}
            onChange={handleGameNameChange}
            placeholder="Enter game name"
            autoFocus
            maxLength={100}
          />
        </FormGroup>

        <FormGroup>
          <Label htmlFor="filePath">Executable Path</Label>
          <TextArea
            id="filePath"
            value={selectedFilePath || ''}
            placeholder="No file selected"
            readOnly
          />
        </FormGroup>

        {errorMessage && errorMessage.length !== 0 && (
          <ErrorText>{errorMessage}</ErrorText>
        )}

        <ModalButtons>
          <Button
            type="button"
            variant="secondary"
            onClick={handleCancel}
            leftIcon={MdClose}
          >
            Cancel
          </Button>
          <Button type="button" variant="primary" onClick={handleAddGame}>
            Add Game
          </Button>
        </ModalButtons>
      </ModalContentContainer>
    </Modal>
  );
};
