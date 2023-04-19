import * as React from 'react';
import styled from 'styled-components';
import { invoke, dialog } from '@tauri-apps/api';
import { FiRefreshCcw } from 'react-icons/fi';
import { FaFolderPlus } from 'react-icons/fa';
import Page from '../../common/page';
import SearchBar from '../../common/searchBar';
import Button from '../../common/button';
import { useGames } from '../../global/contexts/gamesProvider';
import { MonarchGame } from '../search';
import GameCard from '../../common/gameCard';
import spinner from '../../assets/spinner.gif';

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

const LoadingContainer = styled.div`
  position: absolute;
  top: 40%;
  left: 50%;
  text-align: center;
`;

const Spinner = styled.img`
  margin-left: auto;
  margin-right: auto;
`;

const LoadingText = styled.p``;

const Library = () => {
  const { library, updateLibrary } = useGames();
  const [error, setError] = React.useState(false);
  const [searchTerm, setSearchTerm] = React.useState('');
  const [loading, setLoading] = React.useState(false);

  const handleRefresh = React.useCallback(async () => {
    try {
      setLoading(true);

      const result: MonarchGame[] = await invoke('refresh_library');
      updateLibrary(result);

      setLoading(false);
      setError(false);
    } catch (err) {
      setLoading(false);
      setError(true);
    }
  }, [updateLibrary]);

  const handleOpenDialog = React.useCallback(async () => {
    try {
      const path = await dialog.open({
        multiple: false,
        title: 'Choose a game folder',
        directory: true,
      });

      // TODO: Invoke function that adds folder
      await invoke('add_folder', { path });
      setError(false);
    } catch (err) {
      setError(true);
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
          buttonDisabled
        />
        <Button type="button" variant="primary" onClick={handleRefresh}>
          <FiRefreshCcw />
        </Button>
        <Button type="button" variant="primary" onClick={handleOpenDialog}>
          <FaFolderPlus />
        </Button>
      </Row>
      <LibraryContainer>
        {loading ? (
          <LoadingContainer>
            <Spinner src={spinner} alt="loading-spinner" />
            <LoadingText>
              Refreshing your library, this may take a while...
            </LoadingText>
          </LoadingContainer>
        ) : (
          filteredLibrary.map((game) => <GameCard key={game.id} {...game} />)
        )}
        {!loading && error && <p>Something went wrong</p>}
      </LibraryContainer>
    </Page>
  );
};

export default Library;
