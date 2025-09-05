import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import SearchBar from '@_ui/searchBar';
import Spinner from '@_ui/spinner';
import { useSearchGames } from '@global/contexts/searchGamesProvider';
import { Switch } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';
import styled from 'styled-components';

const ResultsContainer = styled.div`
  width: 100%;
  height: calc(100% - 10rem);
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 1rem 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
`;

const MonarchSwitch = styled(Switch)`
  input:checked + .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.primary};
    border-color: ${({ theme }) => theme.colors.primary};
  }

  .mantine-Switch-track {
    background-color: ${({ theme }) => theme.colors.secondary};
    border-color: ${({ theme }) => theme.colors.secondary};
  }

  .mantine-Switch-label {
    color: ${({ theme }) => theme.colors.white};
  }

  &:hover {
    opacity: 0.9;
  }

  display: flex;
  align-items: center;
  margin-bottom: 0rem;
  margin-left: 1rem;

  label {
    margin-left: 1 rem;
    user-select: none;
  }
`;

const SearchRow = styled.div`
  display: flex;
  align-items: center;
  margin-bottom: 1rem;

  @media (max-width: 600px) {
    flex-direction: column;
    align-items: stretch;
    gap: 1.5rem; // Adds space between all flex items
  }
`;

const Search = () => {
  const [searchString, setSearchString] = React.useState('');
  const [reloadKeys, setReloadKeys] = React.useState<Record<string, number>>({});
  const [searchOnMonarch, setSearchOnMonarch] = React.useState(true);
  const { searchedGames, loading, error, searchGames, results } =
    useSearchGames();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchString(e.target.value);
    },
    [],
  );

  const handleSwitchChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchOnMonarch(e.target.checked);
    },
    [],
  );

  const handleClick = React.useCallback(async () => {
    // Reset per-game reload keys for a fresh search
    setReloadKeys({});
    await searchGames(searchString, searchOnMonarch);
  }, [searchGames, searchString, searchOnMonarch]);

  React.useEffect(() => {
    let cancelled = false;

    if (!searchedGames || searchedGames.length === 0) {
      return () => {
        cancelled = true;
      };
    }

    searchedGames.forEach((game) => {
      (async () => {
        try {
          await invoke('download_thumbnail', { game });
          if (cancelled) return;
          setReloadKeys((prev) => ({ ...prev, [game.id]: (prev[game.id] || 0) + 1 }));
        } catch (e) {
          // ignore individual failures; no reload key bump
        }
      })();
    });
    return () => {
      cancelled = true;
    };
  }, [searchedGames]);

  return (
    <Page>
      <SearchRow>
        <SearchBar
          value={searchString}
          onChange={handleChange}
          onSearchClick={handleClick}
          placeholder="Search"
          loading={loading}
        />
        <MonarchSwitch
          checked={searchOnMonarch}
          onChange={handleSwitchChange}
          size="md"
          label="Search on monarch-launcher.com"
          labelPosition="right"
        />
      </SearchRow>
      <ResultsContainer>
        {loading ? (
          <Spinner />
        ) : (
          searchedGames.map((game) => (
            <GameCard
              key={game.id}
              id={game.id}
              executablePath={game.executable_path}
              platform={game.platform}
              name={game.name}
              platformId={game.platform_id}
              thumbnailPath={game.thumbnail_path}
              storePage={game.store_page}
              reloadKey={reloadKeys[game.id]}
            />
          ))
        )}
        {!loading && results?.empty && <p>{results.emptyMessage}</p>}
        {!loading && error && (
          <Error description="Couldn't load games" onRetry={handleClick} />
        )}
      </ResultsContainer>
    </Page>
  );
};

export default Search;
