#pragma once
#include "../IrisLangLibrary/include/IrisInterfaces/IIrisClass.h"
#include <vector>
using namespace std;

class FileTag :IIrisClass{
	vector<FileTag> childs;
	FileTag * pPerant = nullptr;
	string name;
	string absoluteDir;
	string ext;
public:
	FileTag();
	~FileTag();
};

