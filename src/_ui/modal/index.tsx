import type { ModalProps } from '@mantine/core';
import { Modal } from '@mantine/core';
import styled from 'styled-components';

const StyledModal = styled(Modal)`
  .mantine-Paper-root {
    overflow-y: hidden;
  }

  .mantine-Modal-content {
    border-radius: 0.5rem;
    border: 2px solid #FA5002;
  }

  .mantine-Modal-header {
    display: block;
    background-color: ${({ theme }) => theme.colors.black};
  }

  .mantine-Modal-body {
    background-color: ${({ theme }) => theme.colors.black};
  }
`;

export default (props: ModalProps) => {
  const { children } = props;

  return <StyledModal {...props}>{children}</StyledModal>;
};
