import * as React from 'react';
import styled, { css } from 'styled-components';
import { dialog } from '@tauri-apps/api';
import { FiRefreshCcw } from 'react-icons/fi';
import { FaFolderPlus, FaFolderOpen } from 'react-icons/fa';
import { MdClose } from 'react-icons/md';
import { useDisclosure } from '@mantine/hooks';
import type { MonarchGame } from '@global/types';
import Modal from '@_ui/modal';
import Page from '@_ui/page';
import SearchBar from '@_ui/searchBar';
import Button from '@_ui/button';
import { useLibrary } from '@global/contexts/libraryProvider';
import GameCard from '@_ui/gameCard';
import Spinner from '@_ui/spinner';
import Error from '@_ui/error';

const LibraryContainer = styled.div`
  width: 100%;
  height: calc(100% - 7rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
`;

const Row = styled.div`
  display: flex;
  gap: 1rem;
`;

const ModalHeaderContainer = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-direction: row;
  gap: 8rem;
  width: 100%;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: ${({ theme }) => theme.colors.primary};
`;

const ModalButtons = styled.div`
  display: flex;
  justify-content: right;
  align-items: center;
  gap: 1rem;
  margin: 1rem 0;
`;

const ModalContentContainer = styled.div``;

const StyledRefreshIcon = styled(FiRefreshCcw)<{ $loading: boolean }>`
  ${({ $loading }) =>
    $loading &&
    css`
      animation: spin-animation 1.2s infinite;
    `}

  @keyframes spin-animation {
    0% {
      transform: rotate(359deg);
    }
    100% {
      transform: rotate(0);
    }
  }
`;

const Library = () => {
  const [searchTerm, setSearchTerm] = React.useState('');
  const [dialogError, setDialogError] = React.useState(false);
  const [collectionName, setCollectionName] = React.useState('');
  const [opened, { open, close }] = useDisclosure(false);

  const { library, loading, error, refreshLibrary, results } = useLibrary();

  const handleOpenDialog = React.useCallback(async () => {
    try {
      setDialogError(false);

      // @ts-ignore
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const path = await dialog.open({
        multiple: false,
        title: 'Choose a game folder',
        directory: true,
      });
      // TODO: Invoke function that adds folder
      // await invoke('add_folder', { path });
    } catch (err) {
      setDialogError(true);
    }
  }, []);

  const handleSearchTermChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const handleCollectionNameChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setCollectionName(e.target.value);
    },
    [],
  );

  const createCollection = React.useCallback(() => {}, []);

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
        <ModalHeader>Creat a new collection</ModalHeader>
        <Button type="button" variant="icon" onClick={close}>
          <MdClose color="black" size={24} />
        </Button>
      </ModalHeaderContainer>
    );
  }, [close]);

  return (
    <Page title="Library">
      <Row>
        <SearchBar
          value={searchTerm}
          onChange={handleSearchTermChange}
          placeholder="Search"
        />
        <Button
          type="button"
          variant="primary"
          onClick={refreshLibrary}
          title={loading ? 'Loading...' : 'Refresh'}
          loading={loading}
        >
          <StyledRefreshIcon $loading={loading} />
        </Button>
        <Button
          type="button"
          variant="primary"
          onClick={open}
          title="Add new collection"
        >
          <FaFolderPlus />
        </Button>
        <Button
          type="button"
          variant="primary"
          onClick={handleOpenDialog}
          title="Add game folder"
        >
          <FaFolderOpen />
        </Button>
        <Modal
          title={modalHeader}
          opened={opened}
          onClose={close}
          centered
          withCloseButton={false}
        >
          <ModalContentContainer>
            <SearchBar
              value={collectionName}
              onChange={handleCollectionNameChange}
              hideSearchButton
              placeholder="Enter name"
            />
            <ModalButtons>
              <Button type="button" variant="secondary" onClick={close}>
                Cancel
              </Button>
              <Button
                type="button"
                variant="primary"
                onClick={createCollection}
              >
                Next
              </Button>
            </ModalButtons>
          </ModalContentContainer>
        </Modal>
      </Row>
      <LibraryContainer>
        {filteredLibrary.length === 0 && loading ? (
          <Spinner />
        ) : (
          filteredLibrary.map((game) => (
            <GameCard
              key={game.id}
              id={game.id}
              executablePath={game.executable_path}
              platform={game.platform}
              name={game.name}
              platformId={game.platform_id}
              thumbnailPath={game.thumbnail_path}
              isLibrary
            />
          ))
        )}
        {!loading && results?.empty && <p>{results.emptyMessage}</p>}
        {error && (
          <Error description="Couldn't load library" onRetry={refreshLibrary} />
        )}
        {dialogError && (
          <Error
            description="Couldn't open file explorer"
            onRetry={handleOpenDialog}
          />
        )}
      </LibraryContainer>
    </Page>
  );
};

export default Library;
