#include "die.hpp"

int main(int argc, char **argv)
{
    DIE::ScanFile("/bin/ls", 1, "");
    return 0;
}