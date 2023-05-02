import Home from '@pages/home';
import Library from '@pages/library';
import MooseTestPage from '@pages/mooseTestPage';
import NotFound from '@pages/notFound';
import Search from '@pages/search';
import Settings from '@pages/settings';
import { MemoryRouter, Route, Routes } from 'react-router-dom';

export default () => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/library" element={<Library />} />
        <Route path="/search" element={<Search />} />
        <Route path="/moose" element={<MooseTestPage />} />
        <Route path="/*" element={<NotFound />} />
      </Routes>
    </MemoryRouter>
  );
};
