#ifndef _H_IRISGSETTERSTATEMENT_
#define _H_IRISGSETTERSTATEMENT_

#include "IrisStatement.h"

class IrisIdentifier;
class IrisGSetterStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pGSetteredVariable = nullptr;

public:

	virtual bool Generate();

	IrisGSetterStatement(IrisIdentifier* pGSetteredVariable);
	virtual ~IrisGSetterStatement();
};

#endif