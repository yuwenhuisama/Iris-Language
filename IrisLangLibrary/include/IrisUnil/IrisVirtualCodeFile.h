#ifndef _H_IRISVIRTUALCODEFILE_
#define _H_IRISVIRTUALCODEFILE_

#include "../IrisCompileConfigure.h"

#include "IrisCompiler.h"
//#include <time.h>
#include <fstream>
using namespace std;

class IrisVirtualCodeFile
{
private:
	typedef char  BYTE;
	typedef short WORD;
	typedef int   DWORD;

private:
	struct _FileHeader {
		BYTE m_szMagicWord[4] = { 'i', 'r', 'c', 'f' };
		DWORD m_nVersion = 0x00000001;
		time_t m_nTimeStamp = 0;
		DWORD m_nStringFieldElemCount = 0;
		DWORD m_nUniqueFieldElemCount = 0;
		DWORD m_nIntegerFieldElemCount = 0;
		DWORD m_nFloatFieldElemCount = 0;
		DWORD m_nIdentifierFieldElemCount = 0;
		DWORD m_nVirtualCodeCount = 0;
	};

private:
	_FileHeader m_fhHeader;
	//IrisCompiler::_StatementInfo* m_pInfo = nullptr;
	bool _WriteFileBody(fstream& fOutFile, const IrisCompiler::_StatementInfo* pInfo);
	bool _WriteString(fstream& fOutFile, const string& strInput);

	bool _ReadFileBody(fstream& fInFile, IrisCompiler::_StatementInfo* pInfo);
	bool _ReadString(fstream& fInFile, string& strGet);

public:

	bool SaveToFile(const string& strOrgFileName, const IrisCompiler::_StatementInfo* pInfo);

	bool LoadFromFile(const string& strOrgFileName, IrisCompiler::_StatementInfo* pInfo);

	IrisVirtualCodeFile();
	~IrisVirtualCodeFile();
};

#endif