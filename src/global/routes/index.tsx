import { Route, Routes, MemoryRouter } from 'react-router-dom';
import Home from '../../pages/home';
import Settings from '../../pages/settings';
import Library from '../../pages/library';
import Search from '../../pages/search';
import MooseTestPage from '../../pages/mooseTestPage';
import NotFound from '../../pages/notFound';

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
