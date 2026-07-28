#ifndef _H_IRISCODESEGMENT_
#define _H_IRISCODESEGMENT_

#include "IrisComponents/IrisVirtualCodeStructures.h"
#include <vector>
#include <string>
using namespace std;

class IrisCodeSegment
{
public:
	int m_nBelongingFileIndex = -1;

	vector<IR_WORD>* m_pWholeCodes = nullptr;
	unsigned m_nStartPointer = 0;
	unsigned m_nEndPointer = 0;

public:
	IrisCodeSegment();
	~IrisCodeSegment();
};

#endif