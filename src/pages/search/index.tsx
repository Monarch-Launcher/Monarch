import Button from '@_ui/button';
import Error from '@_ui/error';
import GameCard from '@_ui/gameCard';
import Modal from '@_ui/modal';
import Page from '@_ui/page';
import Spinner from '@_ui/spinner';
import { useSearchGames } from '@global/contexts/searchGamesProvider';
import { SearchFilter } from '@global/types';
import {
  Checkbox,
  Divider,
  Group,
  Stack,
  Switch,
} from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import * as React from 'react';
import { AiOutlineSearch } from 'react-icons/ai';
import { FaFilter } from 'react-icons/fa';
import styled from 'styled-components';

const SearchContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 900px;
  margin: 3rem auto 2rem;
  padding: 0 2rem;

  @media (max-width: 768px) {
    margin: 2rem auto 1.5rem;
    padding: 0 1rem;
  }
`;

const SearchBarWrapper = styled.div`
  width: 100%;
  display: flex;
  align-items: center;
  gap: 1.5rem;
  margin-bottom: 1.5rem;

  @media (max-width: 768px) {
    gap: 1rem;
  }
`;

const SearchIconButton = styled.button`
  background: ${({ theme }) => theme.colors.primary};
  border: none;
  border-radius: 0.5rem;
  width: 4rem;
  height: 4rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;

  svg {
    color: ${({ theme }) => theme.colors.black};
  }

  &:hover {
    background: ${({ theme }) => theme.colors.primary};
    transform: scale(1.05);
    box-shadow: 0 0 20px ${({ theme }) => theme.colors.glowOrange};
  }

  &:active {
    transform: scale(0.95);
  }

  @media (max-width: 768px) {
    width: 3.5rem;
    height: 3.5rem;
  }
`;

const SearchInput = styled.input`
  flex: 1;
  background: ${({ theme }) => theme.colors.surface};
  border: 2px solid ${({ theme }) => theme.colors.border};
  border-radius: 0.75rem;
  color: ${({ theme }) => theme.colors.white};
  font-size: 1.25rem;
  padding: 1.25rem 1.75rem;
  transition: all 0.2s ease;

  &::placeholder {
    color: ${({ theme }) => theme.colors.textSecondary};
  }

  &:focus {
    outline: none;
    border-color: ${({ theme }) => theme.colors.primary};
    box-shadow: 0 0 0 3px ${({ theme }) => theme.colors.glowOrange};
  }

  @media (max-width: 768px) {
    font-size: 1.1rem;
    padding: 1rem 1.5rem;
  }
`;

const ButtonRow = styled.div`
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;

  @media (max-width: 600px) {
    width: 100%;
    justify-content: stretch;

    button {
      flex: 1;
      min-width: 120px;
    }
  }
`;

const ResultsContainer = styled.div`
  width: 100%;
  flex: 1;
  overflow-y: auto;
  border-radius: 0.5rem;
  margin: 0;
  padding: 1rem 2rem;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-content: flex-start;

  @media (max-width: 768px) {
    padding: 1rem;
  }
`;

const ModalTitle = styled.span`
  color: #fff;
  font-size: 2rem;
  font-weight: 700;
`;

const SectionTitle = styled.h3`
  color: #fff;
  font-size: 1.4rem;
  font-weight: 600;
  margin: 0;
`;

const StyledCheckbox = styled(Checkbox)`
  .mantine-Checkbox-label {
    color: ${({ theme }) => theme.colors.white};
    cursor: pointer;
  }
  .mantine-Checkbox-input {
    background-color: ${({ theme }) => theme.colors.surface};
    border-color: ${({ theme }) => theme.colors.border};
    cursor: pointer;
    &:checked {
      background-color: ${({ theme }) => theme.colors.primary};
      border-color: ${({ theme }) => theme.colors.primary};
    }
  }
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
    font-family: 'IBM Plex Mono', Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 1rem;
    font-weight: 500;
  }

  &:hover {
    opacity: 0.9;
  }
`;

const Search = () => {
  const [searchString, setSearchString] = React.useState('');
  const [reloadKeys, setReloadKeys] = React.useState<Record<string, number>>({});
  const [searchFilter, setSearchFilter] = React.useState<SearchFilter>({
    steam: true,
    epic: true,
    gog: true,
    itch: true,
    monarch: true,
    steam_powered: false,
    egs: false,
  });
  const [filtersOpen, setFiltersOpen] = React.useState(false);

  const { searchedGames, loading, error, searchGames, results } =
    useSearchGames();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearchString(e.target.value);
    },
    [],
  );

  const handleFilterChange = (key: keyof SearchFilter) => {
    setSearchFilter((prev) => {
      const newState = { ...prev, [key]: !prev[key] };

      // Exclusive logic for search sources
      if (key === 'monarch' && newState.monarch) {
        newState.steam_powered = false;
        newState.egs = false;
      } else if ((key === 'steam_powered' || key === 'egs') && newState[key]) {
        newState.monarch = false;
      }

      return newState;
    });
  };

  const handleClick = React.useCallback(async () => {
    // Reset per-game reload keys for a fresh search
    setReloadKeys({});
    await searchGames(searchString, searchFilter);
  }, [searchGames, searchString, searchFilter]);

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
      <SearchContainer>
        <SearchBarWrapper>
          <SearchInput
            value={searchString}
            onChange={handleChange}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleClick();
              }
            }}
            placeholder="Search for games..."
          />
          <SearchIconButton onClick={handleClick} disabled={loading}>
            <AiOutlineSearch size={32} />
          </SearchIconButton>
        </SearchBarWrapper>
        <ButtonRow>
          <Button
            type="button"
            variant="secondary"
            onClick={() => setFiltersOpen(true)}
            leftIcon={FaFilter}
          >
            Filters
          </Button>
          {/* Future buttons can be added here */}
        </ButtonRow>
      </SearchContainer>
      <Modal
        opened={filtersOpen}
        onClose={() => setFiltersOpen(false)}
        title={<ModalTitle>Search Filters</ModalTitle>}
        centered
        withCloseButton={false}
        size="md"
      >
        <Stack spacing="md" p="md">
          <SectionTitle>Stores</SectionTitle>
          <Group>
            <StyledCheckbox
              label="Steam"
              checked={searchFilter.steam}
              onChange={() => handleFilterChange('steam')}
            />
            <StyledCheckbox
              label="Epic"
              checked={searchFilter.epic}
              onChange={() => handleFilterChange('epic')}
            />
            <StyledCheckbox
              label="GOG"
              checked={searchFilter.gog}
              onChange={() => handleFilterChange('gog')}
            />
            <StyledCheckbox
              label="Itch"
              checked={searchFilter.itch}
              onChange={() => handleFilterChange('itch')}
            />
          </Group>

          <Divider color="gray" />

          <SectionTitle>Search Sources</SectionTitle>
          <Stack spacing="xs">
            <MonarchSwitch
              label="monarch-launcher.com"
              checked={searchFilter.monarch}
              onChange={() => handleFilterChange('monarch')}
            />
            <MonarchSwitch
              label="steampowered.com"
              checked={searchFilter.steam_powered}
              onChange={() => handleFilterChange('steam_powered')}
            />
            <MonarchSwitch
              label="epicgames.com"
              checked={searchFilter.egs}
              onChange={() => handleFilterChange('egs')}
            />
          </Stack>
          <Group position="right" mt="md">
            <Button
              type="button"
              variant="primary"
              onClick={() => setFiltersOpen(false)}
            >
              Done
            </Button>
          </Group>
        </Stack>
      </Modal>
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
              thumbnailUrl={game.thumbnail_url || ''}
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
