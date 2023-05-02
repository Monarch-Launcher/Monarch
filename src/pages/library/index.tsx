import Button from '@_ui/button';
import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import SearchBar from '@_ui/searchBar';
import Spinner from '@_ui/spinner';
import { useLibrary } from '@global/contexts/libraryProvider';
import { FaFolderOpen, FaFolderPlus, FiRefreshCcw } from '@global/icons';
import type { MonarchGame } from '@global/types';
import { useDisclosure } from '@mantine/hooks';
import { dialog } from '@tauri-apps/api';
import * as React from 'react';
import styled, { css } from 'styled-components';

import Modal from './modal';

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

  const filteredLibrary = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) =>
      game.name
        .replace(/[.,/#!$%^&*;:{}=\-_`~()]/g, '')
        .toLowerCase()
        .match(searchTerm.toLowerCase()),
    );
  }, [library, searchTerm]);

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
        <Modal opened={opened} close={close} library={library} />
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
