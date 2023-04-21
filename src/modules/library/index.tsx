import * as React from 'react';
import styled, { css } from 'styled-components';
import { dialog } from '@tauri-apps/api';
import { FiRefreshCcw } from 'react-icons/fi';
import { FaFolderPlus } from 'react-icons/fa';
import Page from '../../common/page';
import SearchBar from '../../common/searchBar';
import Button from '../../common/button';
import { useLibrary } from '../../global/contexts/libraryProvider';
import GameCard from '../../common/gameCard';
import Spinner from '../../common/spinner';
import Error from '../../common/error';

const LibraryContainer = styled.div`
  width: 85%;
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
  const { library, loading, error, refreshLibrary, results } = useLibrary();

  const handleOpenDialog = React.useCallback(async () => {
    try {
      const path = await dialog.open({
        multiple: false,
        title: 'Choose a game folder',
        directory: true,
      });

      // TODO: Invoke function that adds folder
      // await invoke('add_folder', { path });
      setDialogError(false);
    } catch (err) {
      setDialogError(true);
    }
  }, []);

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchTerm(e.target.value);
    },
    [],
  );

  const filteredLibrary = React.useMemo(() => {
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
          onChange={handleChange}
          onSearchClick={() => {}}
          placeholder="Search"
        />
        <Button
          type="button"
          variant="primary"
          onClick={refreshLibrary}
          title="Refresh"
        >
          <StyledRefreshIcon $loading={loading} />
        </Button>
        <Button
          type="button"
          variant="primary"
          onClick={handleOpenDialog}
          title="Add game folder"
        >
          <FaFolderPlus />
        </Button>
      </Row>
      <LibraryContainer>
        {filteredLibrary.length === 0 && loading ? (
          <Spinner />
        ) : (
          filteredLibrary.map((game) => <GameCard key={game.id} {...game} />)
        )}
        {!loading && results?.empty && <p>{results.message}</p>}
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
