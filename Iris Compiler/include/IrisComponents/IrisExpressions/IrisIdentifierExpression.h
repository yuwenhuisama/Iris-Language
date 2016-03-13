#ifndef _H_IRISIDENTIFEREXPRESSION_
#define _H_IRISIDENTIFEREXPRESSION_
#include "IrisExpression.h"
#include "IrisComponents/IrisComponentsDefines.h"
#include <string>
using namespace std;

class IrisIdentifier;

class IrisIdentifierExpression :
	public IrisExpression
{

protected:
	IrisIdentifier* m_pIdentifier = nullptr;

public:

	virtual const string& GetIdentifierString();
	virtual bool Generate();

	virtual bool LeftValue(IrisAMType& eType, IR_DWORD& bIndex);

	IrisIdentifierExpression(IrisIdentifier* pIdentifier);
	virtual ~IrisIdentifierExpression();
};

#endif