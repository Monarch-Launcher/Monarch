import Button from '@_ui/button';
import GameCard from '@_ui/gameCard';
import Modal from '@_ui/modal';
import { useLibrary } from '@global/contexts/libraryProvider';
import { BiEdit, MdClose } from '@global/icons';
import type { Collection, MonarchGame } from '@global/types';
import * as React from 'react';
import styled from 'styled-components';

import EditCollectionForm from './editCollection';

const ModalHeaderContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: ${({ theme }) => theme.colors.primary};
`;

const ContentContainer = styled.div``;

const GamesContainer = styled.div`
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  height: 70vh;
  overflow-y: scroll;
`;

const Flex = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
`;

type Props = {
  opened: boolean;
  close: () => void;
  collection: Collection;
};

const CollectionModal = ({ opened, close, collection }: Props) => {
  const [isEditing, setIsEditing] = React.useState(false);
  const { library } = useLibrary();

  const toggleEditing = React.useCallback(() => {
    setIsEditing((prev) => !prev);
  }, []);

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>
          <Flex>
            {!isEditing ? collection.name : `Edit ${collection.name}`}
            {!isEditing && (
              <Button
                type="button"
                variant="primary"
                rightIcon={BiEdit}
                onClick={toggleEditing}
              >
                Edit
              </Button>
            )}
          </Flex>
        </ModalHeader>
        <Button type="button" variant="icon" onClick={close}>
          <MdClose color="black" size={24} />
        </Button>
      </ModalHeaderContainer>
    );
  }, [close, collection.name, isEditing, toggleEditing]);

  const collectionGames = React.useMemo<MonarchGame[]>(() => {
    return library.filter((game) => collection.gameIds.includes(game.id));
  }, [library, collection.gameIds]);

  return (
    <Modal
      opened={opened}
      onClose={close}
      title={modalHeader}
      centered
      withCloseButton={false}
      size="60vw"
    >
      <ContentContainer>
        {!isEditing ? (
          <GamesContainer>
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
          </GamesContainer>
        ) : (
          <EditCollectionForm
            closeCollection={close}
            toggleEditing={toggleEditing}
            collection={collection}
          />
        )}
      </ContentContainer>
    </Modal>
  );
};

export default CollectionModal;
