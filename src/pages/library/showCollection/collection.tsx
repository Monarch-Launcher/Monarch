import { FaFolder, FaFolderOpen } from '@global/icons';
import type { Collection } from '@global/types';
import { useDisclosure } from '@mantine/hooks';
import * as React from 'react';
import styled from 'styled-components';

import CollectionModal from './showCollectionModal';

const Container = styled.div`
  margin: 0.5rem;
  width: 15rem;
  height: 15rem;
`;

const Flex = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  height: 100%;

  > svg {
    fill: ${({ theme }) => theme.colors.primary};
  }

  &:hover {
    cursor: pointer;
  }
`;

const Name = styled.p`
  margin: 0;
  font-weight: 600;
`;

type Props = {
  collection: Collection;
};

export default ({ collection }: Props) => {
  const [isHovering, setIsHovering] = React.useState(false);
  const [opened, { open, close }] = useDisclosure(false);

  const closeFolder = React.useCallback(() => {
    setIsHovering(false);
  }, []);

  const openFolder = React.useCallback(() => {
    setIsHovering(true);
  }, []);

  return (
    <Container>
      <Flex
        title={`Open '${collection.name}'`}
        onMouseOver={openFolder}
        onMouseOut={closeFolder}
        onClick={open}
      >
        {isHovering ? <FaFolderOpen size={128} /> : <FaFolder size={128} />}
        <Name>{collection.name}</Name>
      </Flex>
      <CollectionModal opened={opened} close={close} collection={collection} />
    </Container>
  );
};
