import * as React from 'react';
import styled, { AnyStyledComponent } from 'styled-components';
import Logo from '@assets/logo.svg';

import NavigationMenu from './navigation-menu';

const PageContainer = styled.div`
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column; // Stack items vertically
  overflow-y: hidden;
`;

const ContentContainer = styled.div`
  padding: 0.5rem 1rem;
  width: 100%;
  min-height: 100vh;
  flex: 1; // Makes the content area take up remaining space
  padding-top: 120px;
  overflow-y: visible;
`;

const BottomMenuContainer = styled.div`
  width: 100%;
`;

// TODO: Proper svg setup for styled components
const StyledLogo = styled(Logo as AnyStyledComponent)`
  position: absolute; // Position the logo in the top-left corner
  top: 0;
  left: 0;
  margin: 1rem; // Add some space around the logo
  width: auto; // Allow for natural scaling
  height: 80px; // Adjust this to make it larger
  max-width: none; // Allow the logo to grow larger than its container width (if needed)
  max-height: none; // Remove max height limitation
  .st0 {
    fill: ${({ theme }) => theme.colors.primary};
  }
`;

type PageProps = {
  children: React.ReactNode;
  hideMenu?: boolean;
};

const Page = ({ children, hideMenu = false }: PageProps) => {
  return (
    <PageContainer>
      <ContentContainer>
        <StyledLogo />
        {children}
      </ContentContainer>

      {!hideMenu && (
        <BottomMenuContainer>
          <NavigationMenu />
        </BottomMenuContainer>
      )}
    </PageContainer>
  );
};

export default Page;
