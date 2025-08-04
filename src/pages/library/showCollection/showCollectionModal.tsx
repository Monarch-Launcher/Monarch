import Modal from '@_ui/modal';
import type { Collection } from '@global/types';
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
  color: #fff;
`;

const ContentContainer = styled.div`
  color: #fff;
`;

type Props = {
  opened: boolean;
  close: () => void;
  collection: Collection;
};

const CollectionModal = ({ opened, close, collection }: Props) => {
  // Remove isEditing state and toggleEditing
  // const [isEditing, setIsEditing] = React.useState(false);
  // const toggleEditing = React.useCallback(() => {
  //   setIsEditing((prev) => !prev);
  // }, []);

  const modalHeader = React.useMemo<JSX.Element>(() => {
    return (
      <ModalHeaderContainer>
        <ModalHeader>Edit {collection.name}</ModalHeader>
        {/* Remove close (X) button */}
      </ModalHeaderContainer>
    );
  }, [collection.name]);

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
        <EditCollectionForm
          closeCollection={close}
          toggleEditing={() => {}}
          collection={collection}
        />
      </ContentContainer>
    </Modal>
  );
};

export default CollectionModal;
