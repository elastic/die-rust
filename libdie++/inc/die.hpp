#include <memory>
#include <stdint.h>
#include <string>

#include "die.h"

#ifndef DIELIB_VERSION
#define DIELIB_VERSION "Unknown"
#endif // DIELIB_VERSION

#ifndef DIE_VERSION
#define DIE_VERSION "Unknown"
#endif // DIE_VERSION

namespace DIE
{
    std::unique_ptr<std::string>
    scan_file(std::string const &filename, uint32_t flags, std::string const &db)
    {
        auto res = ::DIE_ScanFileA(const_cast<char *>(filename.data()), static_cast<int>(flags), const_cast<char *>(db.data()));
        if (res == nullptr)
        {
            return nullptr;
        }

        auto const res_str = std::string(res);
        ::DIE_FreeMemoryA(const_cast<char *>(res));
        return std::make_unique<std::string>(res_str);
    }

} // namespace DIE
