import * as React from 'react';
import { Modal } from '@mantine/core';
import type { ModalProps, Styles, ModalBaseStylesNames } from '@mantine/core';
import { colors } from '@global/theme/index';

export default (props: ModalProps) => {
  const { children } = props;

  const modalStyles = React.useMemo<Styles<ModalBaseStylesNames>>(() => {
    return {
      header: {
        backgroundColor: colors.black,
      },
      body: {
        backgroundColor: colors.black,
      },
    };
  }, []);

  return (
    <Modal styles={modalStyles} {...props}>
      {children}
    </Modal>
  );
};
