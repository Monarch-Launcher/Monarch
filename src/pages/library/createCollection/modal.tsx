import Button from '@_ui/button';
import Modal from '@_ui/modal';
import SearchBar from '@_ui/searchBar';
import { useCollections } from '@global/contexts/collectionsProvider';
import {
  AiOutlinePlus,
  BiLeftArrowAlt,
  BiRightArrowAlt,
  MdClose,
} from '@global/icons';
import type { MonarchGame } from '@global/types';
import * as React from 'react';
import styled from 'styled-components';

import GameRow, { OperationEnum } from './gameRow';

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
`;

const ErrorText = styled.p`
  margin: 0.2rem 0 0 0;
  color: ${({ theme }) => theme.colors.error};
`;

const GameContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  height: 50vh;
  overflow-y: scroll;
  color: #fff;
`;

const Section = styled.div`
  margin-bottom: 1rem;
  color: #fff;
`;

type Props = {
  opened: boolean;
  close: () => void;
  library: MonarchGame[];
};

export default ({ opened, close, library }: Props) => {
  const [searchTerm, setSearchTerm] = React.useState('');
  const [nextClicked, setNextClicked] = React.useState(false);
  const [errorMessage, setErrorMessage] = React.useState<string | undefined>();
  const [selectedGames, setSelectedGames] = React.useState<string[]>([]);
  const [collectionName, setCollectionName] = React.useState('');

  const { createCollection } = useCollections();

  const toggleNext = React.useCallback(() => {
    if (collectionName.length === 0) {
      setErrorMessage('Collection name must be at least 1 character.');
      return;
    }
    setNextClicked((prev) => !prev);
  }, [collectionName]);

  const handleCreateCollection = React.useCallback(async () => {
    await createCollection(collectionName, selectedGames);
    close();
    setCollectionName('');
    setNextClicked(false);
  }, [close, collectionName, selectedGames, createCollection]);

  const updateSelectedGames = React.useCallback(
    (id: string, operation: OperationEnum) => {
      if (operation === OperationEnum.ADD) {
        setSelectedGames((prev) => [...prev, id]);
        return;
      }
      setSelectedGames((prev) => prev.filter((el) => el !== id));
    },
    [],
  );

  const handleCollectionNameChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setCollectionName(e.target.value);
      setErrorMessage(undefined);
    },
    [],
  );

  const handleSearchTermChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) =>
      game.name
        .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
        .toLowerCase()
        .match(searchTerm.toLowerCase()),
    );
  }, [library, searchTerm]);

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>
          {nextClicked ? 'Choose games to add' : 'Create new collection'}
        </ModalHeader>
        <Button type="button" variant="icon" onClick={close}>
          <MdClose color="black" size={24} />
        </Button>
      </ModalHeaderContainer>
    );
  }, [close, nextClicked]);

  return (
    <Modal
      title={modalHeader}
      opened={opened}
      onClose={close}
      centered
      withCloseButton={false}
    >
      <ModalContentContainer>
        {!nextClicked ? (
          <>
            <SearchBar
              autoFocus
              value={collectionName}
              onChange={handleCollectionNameChange}
              hideSearchButton
              placeholder="Enter name"
              fullWidth
              maxLength={50}
            />
            {errorMessage && errorMessage.length !== 0 && (
              <ErrorText>{errorMessage}</ErrorText>
            )}
            <ModalButtons>
              <Button
                type="button"
                variant="secondary"
                onClick={close}
                leftIcon={MdClose}
              >
                Cancel
              </Button>
              <Button
                type="button"
                variant="primary"
                onClick={toggleNext}
                rightIcon={BiRightArrowAlt}
              >
                Next
              </Button>
            </ModalButtons>
          </>
        ) : (
          <>
            <SearchBar
              value={searchTerm}
              onChange={handleSearchTermChange}
              placeholder="Search"
            />
            <GameContainer>
              {filteredLibrary.map((game) => (
                <GameRow
                  key={game.id}
                  id={game.id}
                  name={game.name}
                  updateSelectedGames={updateSelectedGames}
                />
              ))}
            </GameContainer>
            <ModalButtons>
              <Button
                type="button"
                variant="secondary"
                onClick={toggleNext}
                leftIcon={BiLeftArrowAlt}
              >
                Back
              </Button>
              <Button
                type="button"
                variant="primary"
                onClick={handleCreateCollection}
                rightIcon={AiOutlinePlus}
              >
                Create
              </Button>
            </ModalButtons>
          </>
        )}
      </ModalContentContainer>
    </Modal>
  );
};
