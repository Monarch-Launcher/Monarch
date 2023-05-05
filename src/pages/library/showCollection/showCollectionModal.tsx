import Button from '@_ui/button';
import GameCard from '@_ui/gameCard';
import Modal from '@_ui/modal';
import { useLibrary } from '@global/contexts/libraryProvider';
import { AiOutlinePlus, BiRename, MdClose } from '@global/icons';
import type { Collection, MonarchGame } from '@global/types';
import * as React from 'react';
import styled from 'styled-components';

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

const ModalButtons = styled.div`
  display: flex;
  justify-content: right;
  align-items: center;
  gap: 1rem;
  margin: 1rem 0 0.5rem;
`;

type Props = {
  opened: boolean;
  close: () => void;
  collection: Collection;
};

const CollectionModal = ({ opened, close, collection }: Props) => {
  const { library } = useLibrary();
  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>{collection.name}</ModalHeader>
        <Button type="button" variant="icon" onClick={close}>
          <MdClose color="black" size={24} />
        </Button>
      </ModalHeaderContainer>
    );
  }, [close, collection.name]);

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
              isLibrary
            />
          ))}
        </GamesContainer>
        <ModalButtons>
          <Button
            type="button"
            variant="primary"
            onClick={close}
            rightIcon={BiRename}
          >
            Edit name
          </Button>
          <Button
            type="button"
            variant="primary"
            onClick={() => {}}
            rightIcon={AiOutlinePlus}
          >
            Add games
          </Button>
        </ModalButtons>
      </ContentContainer>
    </Modal>
  );
};

export default CollectionModal;
