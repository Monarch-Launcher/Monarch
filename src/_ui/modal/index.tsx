import { colors } from '@global/theme/index';
import type { ModalBaseStylesNames, ModalProps, Styles } from '@mantine/core';
import { Modal } from '@mantine/core';
import * as React from 'react';

type Props = ModalProps & {
  maxHeight?: string;
};

export default (props: Props) => {
  const { children, maxHeight } = props;
  const modalStyles = React.useMemo<Styles<ModalBaseStylesNames>>(() => {
    return {
      content: {
        borderRadius: '0.5rem',
      },
      header: {
        backgroundColor: colors.black,
        display: 'block',
      },
      body: {
        backgroundColor: colors.black,
        maxHeight,
      },
    };
  }, [maxHeight]);

  return (
    <Modal styles={modalStyles} {...props}>
      {children}
    </Modal>
  );
};
