import { useLibrary } from '@global/contexts/libraryProvider';
import type { Collection } from '@global/types';
import { useDisclosure } from '@mantine/hooks';
import * as React from 'react';
import styled from 'styled-components';

import CollectionModal from './showCollectionModal';
import GameCard from '@_ui/gameCard';
import { BiEdit } from '@global/icons';

const CollectionTitleRow = styled.div`
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.3rem;
  padding-left: 1rem;
`;

const CollectionTitle = styled.div`
  font-size: 1.35rem;
  font-weight: 800;
  color: #fff;
`;

const EditButton = styled.button`
  background: transparent;
  border: none;
  outline: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 0.2rem;
  margin-left: 0.5rem;
  color: #fff;
  &:hover {
    color: ${({ theme }) => theme.colors.primary};
  }
`;

const RowContainer = styled.div`
  display: flex;
  align-items: flex-start;
  padding: 0.5rem 1rem;
  margin: 0.5rem 3rem 1.5rem 1rem;
  background-color: rgba(34, 34, 34, 0.8);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.5rem;
  transition: background 0.2s;
  flex-direction: row;
  gap: 1rem;
`;

const GamesRow = styled.div`
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
`;

type Props = {
  collection: Collection;
};

export default ({ collection }: Props) => {
  const [opened, { open, close }] = useDisclosure(false);
  const { library } = useLibrary();

  const collectionGames = React.useMemo(() =>
    library.filter((game) => collection.gameIds.includes(game.id)),
    [library, collection.gameIds]
  );

  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    open();
  };

  return (
    <>
      <CollectionTitleRow>
        <CollectionTitle>{collection.name}</CollectionTitle>
        <EditButton title="Edit Collection" onClick={handleEditClick}>
          <BiEdit size={22} />
        </EditButton>
      </CollectionTitleRow>
      <RowContainer title={`Open '${collection.name}'`}>
        <GamesRow>
          {collectionGames.map((game) => (
            <GameCard
              key={game.id}
              id={game.id}
              executablePath={game.executable_path}
              platform={game.platform}
              name={game.name}
              platformId={game.platform_id}
              thumbnailPath={game.thumbnail_path}
              storePage={game.store_page}
              isLibrary
            />
          ))}
        </GamesRow>
      </RowContainer>
      <CollectionModal opened={opened} close={close} collection={collection} />
    </>
  );
};
